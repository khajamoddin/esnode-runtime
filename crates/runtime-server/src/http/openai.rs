use std::{collections::BTreeMap, sync::Arc, time::Instant};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response, Sse},
    routing::post,
    Json, Router,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use runtime_core::contract::errors::RuntimeError;
use runtime_core::contract::io::{
    ChatMessage, InferRequest, InferenceParams, ModelInput, ModelOutput, StreamChunk, TokenUsage,
};
use runtime_core::contract::model::ModelSpec;

use crate::batching::BatchScheduler;
use crate::router::RuntimeRouter;

#[derive(Clone)]
pub struct HttpState {
    pub router: Arc<RuntimeRouter>,
    pub model_registry: Arc<dyn ModelRegistry>,
    pub audit: Arc<dyn Audit>,
    pub batching: Arc<BatchScheduler>,
}

#[async_trait::async_trait]
pub trait ModelRegistry: Send + Sync {
    async fn resolve_spec(&self, model_name: &str) -> Result<ModelSpec, RuntimeError>;
}

pub trait Audit: Send + Sync {
    fn record(&self, event: AuditEvent);
}

#[derive(Clone, Debug)]
pub struct AuditEvent {
    pub request_id: String,
    pub model: String,
    pub backend: String,
    pub status: String,
    pub latency_ms: u64,
    pub usage: Option<TokenUsage>,
}

pub fn routes(state: HttpState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(openai_chat_completions))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub stream: bool,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIMessageOut,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIMessageOut {
    pub role: String,
    pub content: String,
}

fn sse_event_json<T: Serialize>(data: &T) -> axum::response::sse::Event {
    axum::response::sse::Event::default().data(serde_json::to_string(data).unwrap())
}

fn sse_done() -> axum::response::sse::Event {
    axum::response::sse::Event::default().data("[DONE]")
}

pub async fn openai_chat_completions(
    State(state): State<HttpState>,
    headers: HeaderMap,
    Json(body): Json<OpenAIChatCompletionRequest>,
) -> Result<Response, (StatusCode, String)> {
    let t0 = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let created = chrono::Utc::now().timestamp();

    let spec = state
        .model_registry
        .resolve_spec(&body.model)
        .await
        .map_err(to_http_err)?;

    let messages: Vec<ChatMessage> = body
        .messages
        .into_iter()
        .map(|m| ChatMessage {
            role: m.role,
            content: m.content,
            name: m.name,
        })
        .collect();

    let mut metadata = BTreeMap::new();
    if let Some(v) = headers.get("x-user") {
        if let Ok(s) = v.to_str() {
            metadata.insert("user".to_string(), s.to_string());
        }
    }

    let infer_req = InferRequest {
        request_id: request_id.clone(),
        model: spec.name.clone(),
        input: ModelInput::Chat { messages },
        params: InferenceParams {
            max_tokens: body.max_tokens,
            temperature: body.temperature,
            top_p: body.top_p,
            top_k: None,
            stop: None,
            seed: None,
            stream: body.stream,
            timeout_ms: Some(60_000),
            batch_hint: None,
        },
        metadata,
    };

    let (backend, model) = state
        .router
        .route_and_load(&spec, &infer_req)
        .await
        .map_err(to_http_err)?;

    if infer_req.params.stream {
        let stream = backend
            .infer_stream(&model, infer_req.clone())
            .await
            .map_err(to_http_err)?;

        let model_name = spec.name.clone();
        let sse_stream = async_stream::stream! {
            let first = serde_json::json!({
                "id": format!("chatcmpl-{}", &request_id),
                "object": "chat.completion.chunk",
                "created": created,
                "model": model_name,
                "choices": [{
                    "index": 0,
                    "delta": { "role": "assistant" },
                    "finish_reason": serde_json::Value::Null
                }]
            });
            yield Ok::<_, std::convert::Infallible>(sse_event_json(&first));

            let mut usage: Option<TokenUsage> = None;

            futures_util::pin_mut!(stream);
            while let Some(item) = stream.next().await {
                match item {
                    Ok(StreamChunk::Delta { delta_text, .. }) => {
                        let chunk = serde_json::json!({
                            "id": format!("chatcmpl-{}", &request_id),
                            "object": "chat.completion.chunk",
                            "created": created,
                            "model": model_name,
                            "choices": [{
                                "index": 0,
                                "delta": { "content": delta_text },
                                "finish_reason": serde_json::Value::Null
                            }]
                        });
                        yield Ok::<_, std::convert::Infallible>(sse_event_json(&chunk));
                    }
                    Ok(StreamChunk::Event { name, data, .. }) => {
                        let chunk = serde_json::json!({
                            "id": format!("chatcmpl-{}", &request_id),
                            "object": "chat.completion.chunk",
                            "created": created,
                            "model": model_name,
                            "choices": [{
                                "index": 0,
                                "delta": { "content": format!("[event:{}]{}", name, data) },
                                "finish_reason": serde_json::Value::Null
                            }]
                        });
                        yield Ok::<_, std::convert::Infallible>(sse_event_json(&chunk));
                    }
                    Ok(StreamChunk::End { usage: u, .. }) => {
                        usage = u;
                        break;
                    }
                    Ok(StreamChunk::Start { .. }) => {
                        // role already emitted
                    }
                    Err(e) => {
                        let err = serde_json::json!({ "error": { "message": e.to_string() } });
                        yield Ok::<_, std::convert::Infallible>(sse_event_json(&err));
                        break;
                    }
                }
            }

            let final_chunk = serde_json::json!({
                "id": format!("chatcmpl-{}", &request_id),
                "object": "chat.completion.chunk",
                "created": created,
                "model": model_name,
                "choices": [{
                    "index": 0,
                    "delta": {},
                    "finish_reason": "stop"
                }]
            });
            yield Ok::<_, std::convert::Infallible>(sse_event_json(&final_chunk));
            yield Ok::<_, std::convert::Infallible>(sse_done());

            let _ = usage;
        };

        let resp = Sse::new(sse_stream).keep_alive(axum::response::sse::KeepAlive::default());
        return Ok(resp.into_response());
    }

    let core_resp = if !infer_req.params.stream
        && infer_req.params.batch_hint.is_some()
        && backend.capabilities().supports_batching
    {
        let key = format!("{}:{}:{}", backend.name(), spec.name, spec.version);
        state
            .batching
            .submit(key, backend.clone(), model.clone(), infer_req)
            .await
            .map_err(to_http_err)?
    } else {
        backend.infer(&model, infer_req).await.map_err(to_http_err)?
    };

    let assistant_text = match core_resp.output {
        ModelOutput::Chat { message } => message.content,
        ModelOutput::Completion { text } => text,
    };

    let out = OpenAIChatCompletionResponse {
        id: format!("chatcmpl-{}", request_id),
        object: "chat.completion".to_string(),
        created,
        model: spec.name.clone(),
        choices: vec![OpenAIChoice {
            index: 0,
            message: OpenAIMessageOut {
                role: "assistant".to_string(),
                content: assistant_text,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: core_resp.usage.clone(),
    };

    state.audit.record(AuditEvent {
        request_id: core_resp.request_id,
        model: spec.name.clone(),
        backend: backend.name().to_string(),
        status: "ok".to_string(),
        latency_ms: t0.elapsed().as_millis() as u64,
        usage: core_resp.usage,
    });

    Ok(Json(out).into_response())
}

fn to_http_err(e: RuntimeError) -> (StatusCode, String) {
    use runtime_core::contract::errors::RuntimeError::*;
    match e {
        NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        Invalid(msg) => (StatusCode::BAD_REQUEST, msg),
        Unsupported(msg) => (StatusCode::NOT_IMPLEMENTED, msg),
        Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
    }
}
