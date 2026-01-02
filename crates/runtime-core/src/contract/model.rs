use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ModelId(pub String);

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: String,
    pub checksum: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelSpec {
    pub name: String,
    pub version: String,

    /// Where to load from (local path, S3, registry URL, etc.)
    pub source: ModelSource,

    /// "auto" | "onnx" | "gguf" | "torchscript" | "hf"
    pub format: ModelFormat,

    /// Backend preference: "auto" or explicit "onnxrt" / "llamacpp" / "torch"
    pub backend: String,

    /// Backend settings: threads, device, quant hints, session options, etc.
    pub backend_settings: serde_json::Value,

    /// Labels for routing/AB-testing/policy selection.
    pub labels: BTreeMap<String, String>,

    /// Optional governance hooks.
    pub governance: Option<GovernanceSpec>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ModelSource {
    LocalPath { path: String },
    Http { url: String, sha256: Option<String> },
    Registry { name: String, digest: Option<String> },
}

impl ModelSource {
    pub fn as_local_path(&self) -> Option<&str> {
        match self {
            ModelSource::LocalPath { path } => Some(path),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelFormat {
    Auto,
    Onnx,
    Gguf,
    Torchscript,
    Huggingface,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceSpec {
    pub policy_bundle_path: Option<String>,
    pub pii_redaction: Option<bool>,
    pub audit: Option<bool>,
}
