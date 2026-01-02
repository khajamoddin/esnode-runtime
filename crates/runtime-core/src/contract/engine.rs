use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, pin::Pin, sync::Arc, time::Duration};

use super::{
    caps::{BackendCapabilities, DeviceSpec},
    errors::RuntimeError,
    io::{
        EmbedRequest, EmbedResponse, InferRequest, InferResponse, StreamChunk, TokenUsage,
    },
    model::ModelSpec,
};

/// A stable, backend-agnostic handle to a loaded model.
pub type ModelHandle = Arc<dyn LoadedModel + Send + Sync>;

/// Streaming output type used across all backends.
pub type ChunkStream =
    Pin<Box<dyn Stream<Item = Result<StreamChunk, RuntimeError>> + Send + Sync>>;

/// Minimal interface a loaded model must provide.
#[async_trait]
pub trait LoadedModel: Send + Sync {
    fn model_name(&self) -> &str;
    fn model_version(&self) -> &str;
    fn backend_name(&self) -> &str;
    fn labels(&self) -> &BTreeMap<String, String>; // e.g., {"quant":"Q4", "family":"llama"}

    fn device(&self) -> &DeviceSpec;

    /// Optional: best-effort estimate (used for admission control).
    fn memory_footprint_bytes(&self) -> Option<u64> {
        None
    }

    /// Optional: can the backend do continuous batching / dynamic batching?
    fn supports_batching(&self) -> bool {
        false
    }
}

/// Runtime-wide context available to backends (no heavy deps here).
#[derive(Clone)]
pub struct BackendContext {
    pub request_timeout: Duration,
    pub default_headers: BTreeMap<String, String>,
    pub telemetry: Arc<dyn TelemetrySink + Send + Sync>,
}

/// Minimal telemetry contract to keep backends consistent.
/// Server can implement this with OTel/Prometheus/logging.
pub trait TelemetrySink: Send + Sync {
    fn counter(&self, name: &str, value: u64, attrs: &[(&str, &str)]);
    fn histogram(&self, name: &str, value: f64, attrs: &[(&str, &str)]);
    fn event(&self, name: &str, attrs: &[(&str, &str)]);
}

/// Backends implement this trait. This is the core "universal runtime" contract.
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Backend identity and feature set.
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn capabilities(&self) -> BackendCapabilities;

    /// Lightweight validation before load (format check, files present).
    async fn validate(&self, spec: &ModelSpec) -> Result<(), RuntimeError>;

    /// Load model into memory / initialize session.
    async fn load(&self, spec: &ModelSpec, ctx: BackendContext) -> Result<ModelHandle, RuntimeError>;

    /// Unload / release model resources.
    async fn unload(&self, model: &ModelHandle) -> Result<(), RuntimeError>;

    /// Non-stream inference (chat/completions/etc).
    async fn infer(&self, model: &ModelHandle, req: InferRequest)
        -> Result<InferResponse, RuntimeError>;

    /// Stream inference (token streaming, partial chunks). Must be consistent across backends.
    async fn infer_stream(
        &self,
        model: &ModelHandle,
        req: InferRequest,
    ) -> Result<ChunkStream, RuntimeError>;

    /// Optional embeddings.
    async fn embed(&self, model: &ModelHandle, req: EmbedRequest) -> Result<EmbedResponse, RuntimeError> {
        let _ = (model, req);
        Err(RuntimeError::Unsupported("embed".into()))
    }

    /// Optional: runtime introspection (KV cache, warmup state, etc.)
    async fn inspect(
        &self,
        _model: &ModelHandle,
    ) -> Result<BTreeMap<String, serde_json::Value>, RuntimeError> {
        Ok(BTreeMap::new())
    }
}

/// A factory that produces backends, used for static registration or dynamic plugins.
pub trait BackendFactory: Send + Sync {
    fn backend_type(&self) -> &str; // "onnxrt" | "llamacpp" | "torch"
    fn create(&self, settings: serde_json::Value) -> Result<Arc<dyn InferenceBackend>, RuntimeError>;
}

/// Admission control contract (optional hook in server/router).
#[async_trait]
pub trait AdmissionController: Send + Sync {
    async fn allow(&self, model: &ModelHandle, req: &InferRequest) -> Result<(), RuntimeError>;
}

/// Policy hooks (prompt governance, PII redaction, etc.).
#[async_trait]
pub trait PolicyEngine: Send + Sync {
    async fn pre_infer(
        &self,
        model: &ModelHandle,
        req: InferRequest,
    ) -> Result<InferRequest, RuntimeError>;
    async fn post_infer(
        &self,
        model: &ModelHandle,
        resp: InferResponse,
    ) -> Result<InferResponse, RuntimeError>;
    async fn post_stream_chunk(
        &self,
        model: &ModelHandle,
        chunk: StreamChunk,
    ) -> Result<StreamChunk, RuntimeError>;
}

/// Audit sink hook (who called what, latency, backend, version, policy decisions).
pub trait AuditSink: Send + Sync {
    fn log(&self, record: AuditRecord);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub request_id: String,
    pub model: String,
    pub backend: String,
    pub user: Option<String>,
    pub ip: Option<String>,
    pub status: String,
    pub latency_ms: u64,
    pub usage: Option<TokenUsage>,
    pub extra: BTreeMap<String, serde_json::Value>,
}
