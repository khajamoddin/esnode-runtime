use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub supports_chat: bool,
    pub supports_completion: bool,
    pub supports_embeddings: bool,
    pub supports_streaming: bool,
    pub supports_batching: bool,
    pub supports_kv_cache: bool,

    pub devices: Vec<DeviceKind>,
    pub formats: Vec<String>, // ["onnx","gguf","torchscript","hf"]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Cpu,
    Cuda,
    Metal,
    Vulkan,
    Rocm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub kind: DeviceKind,
    pub id: Option<u32>,         // gpu index
    pub name: Option<String>,    // "NVIDIA A10"
    pub memory_bytes: Option<u64>,
}
