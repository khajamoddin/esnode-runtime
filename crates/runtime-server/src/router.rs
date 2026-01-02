use std::{sync::Arc, time::Instant};

use sha2::{Digest, Sha256};

use runtime_core::contract::{
    caps::{BackendCapabilities, DeviceKind, DeviceSpec},
    engine::{BackendContext, InferenceBackend, ModelHandle, TelemetrySink},
    errors::RuntimeError,
    io::{InferRequest, ModelInput},
    model::{ModelFormat, ModelSource, ModelSpec},
};

use crate::cache::LruModelCache;

#[derive(Clone)]
pub struct RuntimeRouter {
    pub backends: Arc<dyn BackendCatalog>,
    pub model_cache: Arc<LruModelCache>,
    pub telemetry: Arc<dyn TelemetrySink + Send + Sync>,
}

#[async_trait::async_trait]
pub trait BackendCatalog: Send + Sync {
    async fn list(&self) -> Vec<Arc<dyn InferenceBackend>>;
}

pub fn resolve_device(spec: &ModelSpec) -> DeviceSpec {
    let s = spec
        .backend_settings
        .get("device")
        .and_then(|v| v.as_str())
        .unwrap_or("cpu");

    if s.starts_with("cuda") {
        let id = s.split(':').nth(1).and_then(|x| x.parse::<u32>().ok());
        DeviceSpec {
            kind: DeviceKind::Cuda,
            id,
            name: None,
            memory_bytes: None,
        }
    } else if s.starts_with("metal") {
        DeviceSpec {
            kind: DeviceKind::Metal,
            id: None,
            name: None,
            memory_bytes: None,
        }
    } else {
        DeviceSpec {
            kind: DeviceKind::Cpu,
            id: None,
            name: None,
            memory_bytes: None,
        }
    }
}

fn required_features(req: &InferRequest) -> Required {
    let mut r = Required {
        wants_chat: false,
        wants_completion: false,
        wants_streaming: req.params.stream,
    };

    match req.input {
        ModelInput::Chat { .. } => r.wants_chat = true,
        ModelInput::Completion { .. } => r.wants_completion = true,
        ModelInput::Raw { .. } => {}
    }
    r
}

#[derive(Debug, Clone)]
struct Required {
    wants_chat: bool,
    wants_completion: bool,
    wants_streaming: bool,
}

pub fn normalize_format(spec: &ModelSpec) -> ModelFormat {
    if spec.format != ModelFormat::Auto {
        return spec.format.clone();
    }
    match &spec.source {
        ModelSource::LocalPath { path } => {
            let p = path.to_lowercase();
            if p.ends_with(".onnx") {
                ModelFormat::Onnx
            } else if p.ends_with(".gguf") {
                ModelFormat::Gguf
            } else if p.ends_with(".pt") || p.ends_with(".torchscript") {
                ModelFormat::Torchscript
            } else {
                ModelFormat::Huggingface
            }
        }
        ModelSource::Http { url, .. } => {
            let u = url.to_lowercase();
            if u.ends_with(".onnx") {
                ModelFormat::Onnx
            } else if u.ends_with(".gguf") {
                ModelFormat::Gguf
            } else {
                ModelFormat::Huggingface
            }
        }
        ModelSource::Registry { .. } => ModelFormat::Huggingface,
    }
}

fn caps_match(
    caps: &BackendCapabilities,
    fmt: &ModelFormat,
    device: &DeviceSpec,
    req: &Required,
) -> bool {
    let fmt_ok = match fmt {
        ModelFormat::Onnx => caps.formats.iter().any(|f| f == "onnx"),
        ModelFormat::Gguf => caps.formats.iter().any(|f| f == "gguf"),
        ModelFormat::Torchscript => caps.formats.iter().any(|f| f == "torchscript"),
        ModelFormat::Huggingface => caps
            .formats
            .iter()
            .any(|f| f == "hf" || f == "huggingface"),
        ModelFormat::Auto => true,
    };

    let dev_ok = caps.devices.iter().any(|d| match (d, &device.kind) {
        (DeviceKind::Cpu, DeviceKind::Cpu) => true,
        (DeviceKind::Cuda, DeviceKind::Cuda) => true,
        (DeviceKind::Metal, DeviceKind::Metal) => true,
        (DeviceKind::Vulkan, DeviceKind::Vulkan) => true,
        (DeviceKind::Rocm, DeviceKind::Rocm) => true,
        _ => false,
    });

    let task_ok = (!req.wants_chat || caps.supports_chat)
        && (!req.wants_completion || caps.supports_completion)
        && (!req.wants_streaming || caps.supports_streaming);

    fmt_ok && dev_ok && task_ok
}

fn score_backend(
    backend: &dyn InferenceBackend,
    fmt: &ModelFormat,
    device: &DeviceSpec,
    prefer: &[String],
) -> i32 {
    let name = backend.name();
    let mut score = 0;

    if let Some(pos) = prefer.iter().position(|x| x == name) {
        score += 1000 - (pos as i32);
    }

    match fmt {
        ModelFormat::Gguf if name == "llamacpp" => score += 200,
        ModelFormat::Onnx if name == "onnxrt" => score += 200,
        ModelFormat::Torchscript if name == "torch" => score += 200,
        _ => {}
    }

    let caps = backend.capabilities();
    match device.kind {
        DeviceKind::Cuda if caps.devices.iter().any(|d| matches!(d, DeviceKind::Cuda)) => score += 50,
        DeviceKind::Metal if caps.devices.iter().any(|d| matches!(d, DeviceKind::Metal)) => score += 50,
        DeviceKind::Cpu => score += 10,
        _ => {}
    }

    if caps.supports_streaming {
        score += 10;
    }

    score
}

impl RuntimeRouter {
    pub async fn route_and_load(
        &self,
        spec: &ModelSpec,
        req: &InferRequest,
    ) -> Result<(Arc<dyn InferenceBackend>, ModelHandle), RuntimeError> {
        let t0 = Instant::now();
        let fmt = normalize_format(spec);
        let device = resolve_device(spec);
        let needed = required_features(req);

        let prefer: Vec<String> = spec
            .backend_settings
            .get("prefer")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let candidates = self.backends.list().await;
        let mut eligible: Vec<Arc<dyn InferenceBackend>> = vec![];

        if spec.backend != "auto" {
            if let Some(b) = candidates.iter().find(|b| b.name() == spec.backend).cloned() {
                if !caps_match(&b.capabilities(), &fmt, &device, &needed) {
                    return Err(RuntimeError::Invalid(format!(
                        "backend '{}' cannot serve format={:?} device={:?} (caps mismatch)",
                        spec.backend, fmt, device.kind
                    )));
                }
                eligible.push(b);
            } else {
                return Err(RuntimeError::NotFound(format!(
                    "backend not available: {}",
                    spec.backend
                )));
            }
        } else {
            for b in candidates {
                if caps_match(&b.capabilities(), &fmt, &device, &needed) {
                    eligible.push(b);
                }
            }
            if eligible.is_empty() {
                return Err(RuntimeError::Invalid(format!(
                    "no backend can serve format={:?} device={:?} streaming={} chat={} completion={}",
                    fmt,
                    device.kind,
                    needed.wants_streaming,
                    needed.wants_chat,
                    needed.wants_completion
                )));
            }

            eligible.sort_by_key(|b| -score_backend(b.as_ref(), &fmt, &device, &prefer));
        }

        let chosen = eligible[0].clone();

        let device_s = format!("{:?}:{:?}", device.kind, device.id);
        let settings_fingerprint = {
            let bytes = serde_json::to_vec(&spec.backend_settings).unwrap_or_default();
            let mut h = Sha256::new();
            h.update(bytes);
            hex::encode(h.finalize())
        };

        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            spec.name,
            spec.version,
            chosen.name(),
            device_s,
            settings_fingerprint
        );
        let backend_ctx = BackendContext {
            request_timeout: std::time::Duration::from_millis(req.params.timeout_ms.unwrap_or(60_000)),
            default_headers: Default::default(),
            telemetry: self.telemetry.clone(),
        };

        let spec_clone = spec.clone();
        let chosen_clone = chosen.clone();

        let model = self
            .model_cache
            .get_or_load(
                &cache_key,
                chosen.clone(),
                move || async move {
                    chosen_clone.validate(&spec_clone).await?;
                    chosen_clone.load(&spec_clone, backend_ctx).await
                },
            )
            .await?;

        self.telemetry.histogram(
            "esnode_router_select_ms",
            t0.elapsed().as_secs_f64() * 1000.0,
            &[("backend", chosen.name()), ("format", &format!("{:?}", fmt))],
        );

        Ok((chosen, model))
    }
}
