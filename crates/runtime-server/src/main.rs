use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_stream::{self as stream, Stream};

use runtime_server::{http, registry};

use runtime_core::contract::caps::{BackendCapabilities, DeviceKind, DeviceSpec};
use runtime_core::contract::engine::{BackendContext, InferenceBackend, ModelHandle, TelemetrySink};
use runtime_core::contract::errors::RuntimeError;
use runtime_core::contract::io::{InferRequest, InferResponse, ModelInput, ModelOutput, StreamChunk};
use runtime_core::contract::model::ModelSpec;
use runtime_server::batching::BatchScheduler;
use runtime_server::cache::{CacheLimits, LruModelCache};
use runtime_server::http::openai::{Audit, AuditEvent, HttpState, ModelRegistry};
use runtime_server::router::{BackendCatalog, RuntimeRouter};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Deserialize)]
struct ChatCompletionRequest {
    model: String,
    stream: Option<bool>,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<ChatChoice>,
}

#[derive(Serialize)]
struct ChatChoice {
    index: u32,
    message: ChatMessageOut,
    finish_reason: String,
}

#[derive(Serialize)]
struct ChatMessageOut {
    role: String,
    content: String,
}

async fn healthz() -> StatusCode {
    StatusCode::OK
}

async fn readyz() -> StatusCode {
    StatusCode::OK
}

async fn chat_completions(Json(req): Json<ChatCompletionRequest>) -> impl IntoResponse {
    let _ = req.messages.len();
    let created = unix_timestamp();
    if req.stream.unwrap_or(false) {
        let stream = sse_stream(req.model.clone(), created);
        return Sse::new(stream).into_response();
    }

    let resp = ChatCompletionResponse {
        id: "cmpl-stub".to_string(),
        object: "chat.completion".to_string(),
        created,
        model: req.model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessageOut {
                role: "assistant".to_string(),
                content: "stub response".to_string(),
            },
            finish_reason: "stop".to_string(),
        }],
    };

    (StatusCode::OK, Json(resp)).into_response()
}

fn sse_stream(model: String, created: i64) -> impl Stream<Item = Result<Event, Infallible>> {
    let chunks = vec![
        Ok(Event::default().data(
            json!({
                "id": "cmpl-stub",
                "object": "chat.completion.chunk",
                "created": created,
                "model": model,
                "choices": [{
                    "index": 0,
                    "delta": {"role": "assistant"},
                    "finish_reason": null
                }]
            })
            .to_string(),
        )),
        Ok(Event::default().data(
            json!({
                "id": "cmpl-stub",
                "object": "chat.completion.chunk",
                "created": created,
                "model": model,
                "choices": [{
                    "index": 0,
                    "delta": {"content": "stub stream response"},
                    "finish_reason": null
                }]
            })
            .to_string(),
        )),
        Ok(Event::default().data("[DONE]")),
    ];

    stream::iter(chunks)
}

fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[tokio::main]
async fn main() {
    let registry = std::sync::Arc::new(registry::InMemoryModelRegistry::new());
    let bundle_registry = std::sync::Arc::new(registry::BundleModelRegistry::new("bundles"));
    let telemetry = Arc::new(StubTelemetry);
    let backend_catalog = Arc::new(InMemoryBackendCatalog::new(vec![Arc::new(StubBackend)]));
    let model_cache = Arc::new(LruModelCache::new(CacheLimits::default()));
    let router = Arc::new(RuntimeRouter {
        backends: backend_catalog,
        model_cache,
        telemetry: telemetry.clone(),
    });
    let model_registry = Arc::new(InMemorySpecRegistry::new(registry.clone()));
    let audit = Arc::new(NoopAudit);
    let batching = Arc::new(BatchScheduler::default());
    let openai_state = HttpState {
        router,
        model_registry,
        audit,
        batching,
    };

    let app = Router::new()
        .merge(http::router(bundle_registry.clone()))
        .merge(http::openai::routes(openai_state))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));

    let addr = "0.0.0.0:9090";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("runtime-server listening on {}", addr);

    let http_server = axum::serve(listener, app);

    #[cfg(feature = "proto-gen")]
    {
        use runtime_server::grpc;
        let grpc_addr = "0.0.0.0:9091".parse().unwrap();
        let grpc_server = tonic::transport::Server::builder()
            .add_service(grpc::service_with_registry(bundle_registry))
            .serve(grpc_addr);

        if let Err(err) = tokio::try_join!(grpc_server, http_server) {
            eprintln!("server error: {err}");
        }
    }

    #[cfg(not(feature = "proto-gen"))]
    {
        if let Err(err) = http_server.await {
            eprintln!("http server error: {err}");
        }
    }
}

struct StubTelemetry;

impl TelemetrySink for StubTelemetry {
    fn counter(&self, _name: &str, _value: u64, _attrs: &[(&str, &str)]) {}
    fn histogram(&self, _name: &str, _value: f64, _attrs: &[(&str, &str)]) {}
    fn event(&self, _name: &str, _attrs: &[(&str, &str)]) {}
}

struct InMemoryBackendCatalog {
    backends: Vec<Arc<dyn InferenceBackend>>,
}

impl InMemoryBackendCatalog {
    fn new(backends: Vec<Arc<dyn InferenceBackend>>) -> Self {
        Self { backends }
    }
}

#[async_trait::async_trait]
impl BackendCatalog for InMemoryBackendCatalog {
    async fn list(&self) -> Vec<Arc<dyn InferenceBackend>> {
        self.backends.clone()
    }
}

struct InMemorySpecRegistry {
    registry: registry::InMemoryRegistry,
}

impl InMemorySpecRegistry {
    fn new(registry: registry::InMemoryRegistry) -> Self {
        Self { registry }
    }
}

#[async_trait::async_trait]
impl ModelRegistry for InMemorySpecRegistry {
    async fn resolve_spec(&self, model_name: &str) -> Result<ModelSpec, RuntimeError> {
        self.registry
            .get_by_name(model_name)
            .ok_or_else(|| RuntimeError::NotFound(format!("model not found: {model_name}")))
    }
}

struct NoopAudit;

impl Audit for NoopAudit {
    fn record(&self, _event: AuditEvent) {}
}

struct StubBackend;

#[async_trait::async_trait]
impl InferenceBackend for StubBackend {
    fn name(&self) -> &str {
        "stub"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_chat: true,
            supports_completion: true,
            supports_embeddings: false,
            supports_streaming: true,
            supports_batching: false,
            supports_kv_cache: false,
            devices: vec![DeviceKind::Cpu],
            formats: vec!["onnx".to_string(), "gguf".to_string(), "torchscript".to_string(), "hf".to_string()],
        }
    }

    async fn validate(&self, _spec: &ModelSpec) -> Result<(), RuntimeError> {
        Ok(())
    }

    async fn load(&self, spec: &ModelSpec, _ctx: BackendContext) -> Result<ModelHandle, RuntimeError> {
        Ok(Arc::new(StubModel {
            name: spec.name.clone(),
            version: spec.version.clone(),
        }))
    }

    async fn unload(&self, _model: &ModelHandle) -> Result<(), RuntimeError> {
        Ok(())
    }

    async fn infer(&self, _model: &ModelHandle, req: InferRequest) -> Result<InferResponse, RuntimeError> {
        let output = match req.input {
            ModelInput::Chat { .. } => ModelOutput::Chat {
                message: runtime_core::contract::io::ChatMessage {
                    role: "assistant".to_string(),
                    content: "stub response".to_string(),
                    name: None,
                },
            },
            ModelInput::Completion { .. } | ModelInput::Raw { .. } => ModelOutput::Completion {
                text: "stub response".to_string(),
            },
        };

        Ok(InferResponse {
            request_id: req.request_id,
            model: req.model,
            output,
            usage: None,
            metadata: BTreeMap::new(),
        })
    }

    async fn infer_stream(
        &self,
        _model: &ModelHandle,
        req: InferRequest,
    ) -> Result<runtime_core::contract::engine::ChunkStream, RuntimeError> {
        let start = StreamChunk::Start {
            request_id: req.request_id.clone(),
            model: req.model.clone(),
            metadata: BTreeMap::new(),
        };
        let delta = StreamChunk::Delta {
            request_id: req.request_id.clone(),
            delta_text: "stub stream response".to_string(),
        };
        let end = StreamChunk::End {
            request_id: req.request_id,
            usage: None,
        };

        let stream = async_stream::stream! {
            yield Ok(start);
            yield Ok(delta);
            yield Ok(end);
        };

        Ok(Box::pin(stream))
    }
}

struct StubModel {
    name: String,
    version: String,
}

#[async_trait::async_trait]
impl runtime_core::contract::engine::LoadedModel for StubModel {
    fn model_name(&self) -> &str {
        &self.name
    }

    fn model_version(&self) -> &str {
        &self.version
    }

    fn backend_name(&self) -> &str {
        "stub"
    }

    fn labels(&self) -> &BTreeMap<String, String> {
        static EMPTY: once_cell::sync::Lazy<BTreeMap<String, String>> =
            once_cell::sync::Lazy::new(BTreeMap::new);
        &EMPTY
    }

    fn device(&self) -> &DeviceSpec {
        static DEVICE: DeviceSpec = DeviceSpec {
            kind: DeviceKind::Cpu,
            id: None,
            name: None,
            memory_bytes: None,
        };
        &DEVICE
    }
}
