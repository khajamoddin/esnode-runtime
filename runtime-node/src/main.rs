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

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn ready() -> StatusCode {
    StatusCode::OK
}

async fn chat_completions(
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
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

fn sse_stream(
    model: String,
    created: i64,
) -> impl Stream<Item = Result<Event, Infallible>> {
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
    let app = Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/v1/chat/completions", post(chat_completions));

    let addr = "0.0.0.0:9090";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("runtime-node listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
