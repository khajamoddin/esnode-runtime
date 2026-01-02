use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferRequest {
    pub request_id: String,
    pub model: String, // resolved by router / registry
    pub input: ModelInput,

    /// Uniform controls (the server maps these to backend-specific params).
    pub params: InferenceParams,

    /// Caller metadata (auth/user/ip/tenant), used for audit/policy.
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelInput {
    Chat { messages: Vec<ChatMessage> },
    Completion { prompt: String },
    Raw { bytes_b64: String, content_type: String }, // for future multimodal
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,   // "system"|"user"|"assistant"|"tool"
    pub content: String,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferenceParams {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub stop: Option<Vec<String>>,
    pub seed: Option<u64>,

    /// Runtime features
    pub stream: bool,
    pub timeout_ms: Option<u64>,

    /// Performance features
    pub batch_hint: Option<BatchHint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BatchHint {
    pub priority: Option<u8>, // 0-10
    pub latency_sla_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferResponse {
    pub request_id: String,
    pub model: String,
    pub output: ModelOutput,
    pub usage: Option<TokenUsage>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelOutput {
    Chat { message: ChatMessage },
    Completion { text: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamChunk {
    /// Emitted at start (model/backend/version, etc.)
    Start { request_id: String, model: String, metadata: BTreeMap<String, String> },

    /// Partial tokens/text. Keep it simple; the HTTP layer can translate to SSE.
    Delta { request_id: String, delta_text: String },

    /// Optional tool-call / structured deltas later.
    Event { request_id: String, name: String, data: serde_json::Value },

    /// Final response and usage.
    End { request_id: String, usage: Option<TokenUsage> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub request_id: String,
    pub model: String,
    pub texts: Vec<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub request_id: String,
    pub model: String,
    pub vectors: Vec<Vec<f32>>,
}
