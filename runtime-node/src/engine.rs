use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ModelSpec {
    pub model_id: String,          // e.g. "llama3-8b-q4_k_m"
    pub model_path: String,        // local path or mounted volume path
    pub engine: String,            // "llamacpp" in v0
    pub options: HashMap<String, String>, // threads, n_gpu_layers, ctx, etc.
}

#[derive(Clone, Debug)]
pub struct GenerationParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub top_p: f32,
    pub seed: Option<u64>,
    pub stop: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CanonicalMessage {
    pub role: String,    // "system" | "user" | "assistant" | "tool"
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct CanonicalRequest {
    pub request_id: String,
    pub tenant_id: String,
    pub model_id: String,
    pub messages: Vec<CanonicalMessage>,
    pub params: GenerationParams,

    // RAG/context injected by gateway (optional in v0)
    pub context_chunks: Vec<ContextChunk>,

    // hard limits enforced by runtime node (defense-in-depth)
    pub time_budget: Duration,
}

#[derive(Clone, Debug)]
pub struct ContextChunk {
    pub source: String,     // e.g. "qdrant://collection/doc#chunk"
    pub text: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Clone, Debug)]
pub enum StreamEvent {
    Token { text: String },
    Done { usage: Usage },
    Error { code: String, message: String },
}

#[derive(Clone, Debug)]
pub struct EngineHealth {
    pub ready: bool,
    pub model_loaded: bool,
    pub engine: String,
    pub model_id: Option<String>,
}

#[async_trait::async_trait]
pub trait ExecutionEngine: Send + Sync {
    /// Prepare engine, allocate resources, and validate runtime environment.
    async fn init(&self) -> anyhow::Result<()>;

    /// Load or switch model (v0: one model per node is acceptable).
    async fn load_model(&self, spec: ModelSpec) -> anyhow::Result<()>;

    /// Generate a completion as a stream of tokens.
    async fn generate_stream(
        &self,
        req: CanonicalRequest,
    ) -> anyhow::Result<Box<dyn tokio_stream::Stream<Item = StreamEvent> + Send + Unpin>>;

    /// Health endpoint signal for gateway/K8s readiness.
    async fn health(&self) -> EngineHealth;

    /// Graceful shutdown.
    async fn shutdown(&self) -> anyhow::Result<()>;
}
