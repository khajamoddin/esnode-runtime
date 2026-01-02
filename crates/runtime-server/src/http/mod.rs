use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use runtime_core::contract::io::{
    ChatMessage, InferRequest, InferResponse, ModelInput, ModelOutput, StreamChunk,
};
use runtime_core::contract::model::ModelSpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::{self as stream, Stream};

use crate::registry::{BundleModelRegistry, ModelRegistry};

pub mod openai;

#[derive(Serialize)]
struct LoadModelResponse {
    model_handle: String,
}

#[derive(Deserialize)]
struct UnloadModelRequest {
    model_handle: String,
}

#[derive(Serialize)]
struct UnloadModelResponse {
    ok: bool,
}

#[derive(Serialize)]
struct ListModelsResponse {
    models: Vec<ModelSpec>,
}

pub fn router(registry: Arc<BundleModelRegistry>) -> Router {
    Router::new()
        .route("/esnode/v1/models/load", post(load_model))
        .route("/esnode/v1/models/unload", post(unload_model))
        .route("/esnode/v1/models", get(list_models))
        .route("/esnode/v1/infer", post(infer))
        .route("/esnode/v1/infer/stream", post(infer_stream))
        .with_state(registry)
}

async fn load_model(
    State(registry): State<Arc<BundleModelRegistry>>,
    Json(req): Json<ModelSpec>,
) -> impl IntoResponse {
    match registry.resolve_spec(&req.name).await {
        Ok(spec) => Json(LoadModelResponse {
            model_handle: spec.name,
        })
        .into_response(),
        Err(_) => (
            axum::http::StatusCode::NOT_FOUND,
            Json(json_error_with_message("bundle not found")),
        )
            .into_response(),
    }
}

async fn unload_model(
    State(_registry): State<Arc<BundleModelRegistry>>,
    Json(req): Json<UnloadModelRequest>,
) -> Json<UnloadModelResponse> {
    let _ = req;
    let ok = true;
    Json(UnloadModelResponse { ok })
}

async fn list_models(
    State(registry): State<Arc<BundleModelRegistry>>,
) -> Json<ListModelsResponse> {
    let models = registry.list_specs().await.unwrap_or_default();
    Json(ListModelsResponse {
        models,
    })
}

async fn infer(
    State(registry): State<Arc<BundleModelRegistry>>,
    Json(req): Json<InferRequest>,
) -> impl IntoResponse {
    if registry.resolve_spec(&req.model).await.is_err() {
        return (axum::http::StatusCode::NOT_FOUND, Json(json_error())).into_response();
    }

    let (output, model) = match req.input.clone() {
        ModelInput::Chat { .. } => (
            ModelOutput::Chat {
                message: ChatMessage {
                    role: "assistant".to_string(),
                    content: "stub response".to_string(),
                    name: None,
                },
            },
            req.model.clone(),
        ),
        ModelInput::Completion { .. } => (
            ModelOutput::Completion {
                text: "stub response".to_string(),
            },
            req.model.clone(),
        ),
        ModelInput::Raw { .. } => (
            ModelOutput::Completion {
                text: "stub response".to_string(),
            },
            req.model.clone(),
        ),
    };

    Json(InferResponse {
        request_id: req.request_id,
        model,
        output,
        usage: None,
        metadata: BTreeMap::new(),
    })
    .into_response()
}

async fn infer_stream(
    State(registry): State<Arc<BundleModelRegistry>>,
    Json(req): Json<InferRequest>,
) -> impl IntoResponse {
    if registry.resolve_spec(&req.model).await.is_err() {
        return (axum::http::StatusCode::NOT_FOUND, Json(json_error())).into_response();
    }

    let stream = stream_chunks(req.request_id.clone(), req.model.clone());
    Sse::new(stream).into_response()
}

fn stream_chunks(
    request_id: String,
    model: String,
) -> impl Stream<Item = Result<Event, Infallible>> {
    let start = StreamChunk::Start {
        request_id: request_id.clone(),
        model: model.clone(),
        metadata: BTreeMap::new(),
    };
    let delta = StreamChunk::Delta {
        request_id: request_id.clone(),
        delta_text: "stub stream response".to_string(),
    };
    let end = StreamChunk::End {
        request_id,
        usage: None,
    };

    let chunks = vec![start, delta, end]
        .into_iter()
        .map(|chunk| {
            let json = serde_json::to_string(&chunk).unwrap_or_else(|_| "{}".to_string());
            Ok(Event::default().data(json))
        })
        .collect::<Vec<_>>();

    stream::iter(chunks)
}

fn json_error() -> serde_json::Value {
    json_error_with_message("model not found")
}

fn json_error_with_message(message: &str) -> serde_json::Value {
    serde_json::json!({ "error": { "message": message } })
}
