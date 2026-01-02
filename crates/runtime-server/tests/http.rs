use axum::body::Body;
use axum::http::{Request, StatusCode};
use runtime_server::http;
use runtime_server::registry::BundleModelRegistry;
use serde_json::json;
use tower::ServiceExt;
use http_body_util::BodyExt;

#[tokio::test]
async fn esnode_http_load_and_infer() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bundles");
    let registry = std::sync::Arc::new(BundleModelRegistry::new(root));
    let app = http::router(registry);

    let load_body = json!({
        "name": "fixture-model",
        "version": "v0",
        "source": { "kind": "local_path", "path": "models/fixture.gguf" },
        "format": "gguf",
        "backend": "llamacpp",
        "backend_settings": {},
        "labels": {}
    });

    let load_req = Request::builder()
        .method("POST")
        .uri("/esnode/v1/models/load")
        .header("content-type", "application/json")
        .body(Body::from(load_body.to_string()))
        .unwrap();

    let load_resp = app.clone().oneshot(load_req).await.unwrap();
    assert_eq!(load_resp.status(), StatusCode::OK);

    let infer_body = json!({
        "request_id": "req-1",
        "model": "fixture-model",
        "input": {
            "type": "chat",
            "messages": [
                { "role": "user", "content": "Hello" }
            ]
        },
        "params": {
            "stream": false
        },
        "metadata": {}
    });

    let infer_req = Request::builder()
        .method("POST")
        .uri("/esnode/v1/infer")
        .header("content-type", "application/json")
        .body(Body::from(infer_body.to_string()))
        .unwrap();

    let infer_resp = app.clone().oneshot(infer_req).await.unwrap();
    assert_eq!(infer_resp.status(), StatusCode::OK);
    let body = infer_resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("stub response"));
}

#[tokio::test]
async fn esnode_http_stream() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bundles");
    let registry = std::sync::Arc::new(BundleModelRegistry::new(root));
    let app = http::router(registry.clone());

    let load_body = json!({
        "name": "fixture-model",
        "version": "v0",
        "source": { "kind": "local_path", "path": "models/fixture.gguf" },
        "format": "gguf",
        "backend": "llamacpp",
        "backend_settings": {},
        "labels": {}
    });

    let load_req = Request::builder()
        .method("POST")
        .uri("/esnode/v1/models/load")
        .header("content-type", "application/json")
        .body(Body::from(load_body.to_string()))
        .unwrap();

    let load_resp = app.clone().oneshot(load_req).await.unwrap();
    assert_eq!(load_resp.status(), StatusCode::OK);

    let infer_body = json!({
        "request_id": "req-2",
        "model": "fixture-model",
        "input": {
            "type": "chat",
            "messages": [
                { "role": "user", "content": "Hello" }
            ]
        },
        "params": {
            "stream": true
        },
        "metadata": {}
    });

    let infer_req = Request::builder()
        .method("POST")
        .uri("/esnode/v1/infer/stream")
        .header("content-type", "application/json")
        .body(Body::from(infer_body.to_string()))
        .unwrap();

    let infer_resp = app.clone().oneshot(infer_req).await.unwrap();
    assert_eq!(infer_resp.status(), StatusCode::OK);
    let body = infer_resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("stub stream response"));
}
