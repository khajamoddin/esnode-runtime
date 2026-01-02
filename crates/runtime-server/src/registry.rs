use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, Mutex},
};

use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use walkdir::WalkDir;

use runtime_core::contract::{
    errors::RuntimeError,
    model::{ModelSource, ModelSpec},
};

#[async_trait::async_trait]
pub trait ModelRegistry: Send + Sync {
    async fn resolve_spec(&self, model_name: &str) -> Result<ModelSpec, RuntimeError>;
    async fn provenance(&self, model_name: &str) -> Result<ModelProvenance, RuntimeError>;
}

#[derive(Clone, Debug)]
pub struct ModelProvenance {
    pub model_name: String,
    pub bundle_dir: PathBuf,
    pub spec_path: PathBuf,
    pub verified_artifacts: Vec<VerifiedArtifact>,
    pub spec_labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct VerifiedArtifact {
    pub path: PathBuf,
    pub sha256_hex: String,
    pub ok: bool,
    pub expected_sha256_hex: Option<String>,
}

#[derive(Clone)]
pub struct BundleModelRegistry {
    bundles_root: PathBuf,
    cache: Arc<RwLock<BTreeMap<String, (ModelSpec, ModelProvenance)>>>,
}

impl BundleModelRegistry {
    pub fn new(bundles_root: impl Into<PathBuf>) -> Self {
        Self {
            bundles_root: bundles_root.into(),
            cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    fn bundle_dir(&self, model_name: &str) -> PathBuf {
        self.bundles_root.join(model_name)
    }

    fn spec_path(bundle_dir: &Path) -> PathBuf {
        bundle_dir.join("model-spec.yaml")
    }

    async fn load_spec_from_disk(
        &self,
        model_name: &str,
    ) -> Result<(ModelSpec, ModelProvenance), RuntimeError> {
        let bundle_dir = self.bundle_dir(model_name);
        let spec_path = Self::spec_path(&bundle_dir);

        let raw = tokio::fs::read_to_string(&spec_path)
            .await
            .map_err(|e| {
                RuntimeError::NotFound(format!(
                    "model-spec.yaml not found: {} ({e})",
                    spec_path.display()
                ))
            })?;

        let mut spec: ModelSpec =
            serde_yaml::from_str(&raw).map_err(|e| RuntimeError::Invalid(format!("invalid YAML: {e}")))?;

        if spec.name.trim().is_empty() {
            spec.name = model_name.to_string();
        }
        if spec.name != model_name {
            return Err(RuntimeError::Invalid(format!(
                "bundle name '{}' does not match spec.name '{}'",
                model_name, spec.name
            )));
        }

        if let ModelSource::LocalPath { path } = &mut spec.source {
            let p = PathBuf::from(path.as_str());
            if p.is_relative() {
                *path = bundle_dir.join(p).to_string_lossy().to_string();
            }
        }

        let verified_artifacts = verify_source_artifacts(&spec).await?;

        let prov = ModelProvenance {
            model_name: model_name.to_string(),
            bundle_dir,
            spec_path,
            verified_artifacts,
            spec_labels: spec.labels.clone(),
        };

        Ok((spec, prov))
    }

    pub async fn list_specs(&self) -> Result<Vec<ModelSpec>, RuntimeError> {
        let mut models = Vec::new();
        let entries = std::fs::read_dir(&self.bundles_root).map_err(|e| {
            RuntimeError::Internal(format!(
                "failed to read bundles root {}: {e}",
                self.bundles_root.display()
            ))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                if let Ok(spec) = self.resolve_spec(name).await {
                    models.push(spec);
                }
            }
        }

        Ok(models)
    }
}

#[async_trait::async_trait]
impl ModelRegistry for BundleModelRegistry {
    async fn resolve_spec(&self, model_name: &str) -> Result<ModelSpec, RuntimeError> {
        if let Some((spec, _)) = self.cache.read().await.get(model_name).cloned() {
            return Ok(spec);
        }
        let (spec, prov) = self.load_spec_from_disk(model_name).await?;
        self.cache
            .write()
            .await
            .insert(model_name.to_string(), (spec.clone(), prov));
        Ok(spec)
    }

    async fn provenance(&self, model_name: &str) -> Result<ModelProvenance, RuntimeError> {
        if let Some((_, prov)) = self.cache.read().await.get(model_name).cloned() {
            return Ok(prov);
        }
        let (spec, prov) = self.load_spec_from_disk(model_name).await?;
        self.cache
            .write()
            .await
            .insert(model_name.to_string(), (spec, prov.clone()));
        Ok(prov)
    }
}

async fn verify_source_artifacts(spec: &ModelSpec) -> Result<Vec<VerifiedArtifact>, RuntimeError> {
    match &spec.source {
        ModelSource::LocalPath { path } => {
            let p = PathBuf::from(path);
            if !p.exists() {
                return Err(RuntimeError::NotFound(format!(
                    "model artifact not found: {}",
                    p.display()
                )));
            }

            let expected = spec
                .backend_settings
                .get("expected_sha256")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if expected.is_none() {
                return Ok(vec![]);
            }

            if p.is_file() {
                let actual = sha256_file(&p).await?;
                let ok = expected.as_deref().map(|e| eq_hash(&actual, e)).unwrap_or(true);
                return Ok(vec![VerifiedArtifact {
                    path: p,
                    sha256_hex: actual,
                    ok,
                    expected_sha256_hex: expected,
                }]);
            }

            let actual = sha256_dir(&p).await?;
            let ok = expected.as_deref().map(|e| eq_hash(&actual, e)).unwrap_or(true);
            Ok(vec![VerifiedArtifact {
                path: p,
                sha256_hex: actual,
                ok,
                expected_sha256_hex: expected,
            }])
        }

        ModelSource::Http { url, sha256 } => Ok(vec![VerifiedArtifact {
            path: PathBuf::from(url),
            sha256_hex: sha256.clone().unwrap_or_default(),
            ok: true,
            expected_sha256_hex: sha256.clone(),
        }]),

        ModelSource::Registry { name, digest } => Ok(vec![VerifiedArtifact {
            path: PathBuf::from(name),
            sha256_hex: digest.clone().unwrap_or_default(),
            ok: true,
            expected_sha256_hex: digest.clone(),
        }]),
    }
}

fn eq_hash(actual_hex: &str, expected_hex: &str) -> bool {
    actual_hex.trim().eq_ignore_ascii_case(expected_hex.trim())
}

async fn sha256_file(path: &Path) -> Result<String, RuntimeError> {
    let data = tokio::fs::read(path)
        .await
        .map_err(|e| RuntimeError::Internal(format!("read failed {}: {e}", path.display())))?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(hex::encode(h.finalize()))
}

async fn sha256_dir(dir: &Path) -> Result<String, RuntimeError> {
    let mut entries: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    entries.sort();

    let mut h = Sha256::new();
    for file in entries {
        let rel = file.strip_prefix(dir).unwrap_or(&file);
        let rel_s = rel.to_string_lossy();
        let file_hash = sha256_file(&file).await?;

        h.update(rel_s.as_bytes());
        h.update([0u8]);
        h.update(file_hash.as_bytes());
    }
    Ok(hex::encode(h.finalize()))
}

#[derive(Default)]
pub struct InMemoryModelRegistry {
    inner: Mutex<BTreeMap<String, ModelSpec>>,
    counter: AtomicU64,
}

impl InMemoryModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&self, spec: ModelSpec) -> String {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let handle = format!("model-{id}");
        self.inner.lock().unwrap().insert(handle.clone(), spec);
        handle
    }

    pub fn unload(&self, handle: &str) -> bool {
        self.inner.lock().unwrap().remove(handle).is_some()
    }

    pub fn list(&self) -> Vec<ModelSpec> {
        self.inner
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    pub fn get_by_handle(&self, handle: &str) -> Option<ModelSpec> {
        self.inner.lock().unwrap().get(handle).cloned()
    }

    pub fn get_by_name(&self, model_name: &str) -> Option<ModelSpec> {
        self.inner
            .lock()
            .unwrap()
            .values()
            .find(|spec| spec.name == model_name)
            .cloned()
    }

    pub fn has_model_name(&self, model_name: &str) -> bool {
        self.inner
            .lock()
            .unwrap()
            .values()
            .any(|spec| spec.name == model_name)
    }
}

pub type BundleRegistry = Arc<BundleModelRegistry>;
pub type InMemoryRegistry = Arc<InMemoryModelRegistry>;
