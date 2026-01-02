#![cfg(feature = "proto-gen")]

use runtime_server::grpc;
use runtime_server::registry::BundleModelRegistry;
use tokio_stream::StreamExt;
use tonic::Request;

#[tokio::test]
async fn grpc_load_and_infer() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bundles");
    let registry = std::sync::Arc::new(BundleModelRegistry::new(root));
    let service = grpc::RuntimeServiceImpl::new(registry.clone());

    let load_req = grpc::runtime::LoadModelRequest {
        spec: Some(grpc::runtime::ModelSpec {
            name: "fixture-model".to_string(),
            version: "v0".to_string(),
            format: "gguf".to_string(),
            backend: "llamacpp".to_string(),
            source_kind: "local_path".to_string(),
            source: "models/fixture.gguf".to_string(),
            sha256: "".to_string(),
            backend_settings_json: "{}".to_string(),
            labels: Default::default(),
        }),
    };

    let load_resp = service
        .load_model(Request::new(load_req))
        .await
        .unwrap()
        .into_inner();

    let infer_req = grpc::runtime::InferRequest {
        request_id: "req-1".to_string(),
        model_handle: load_resp.model_handle,
        input: Some(grpc::runtime::infer_request::Input::Chat(
            grpc::runtime::ChatInput { messages: vec![] },
        )),
        params: Some(grpc::runtime::InferenceParams {
            max_tokens: 0,
            temperature: 0.0,
            top_p: 0.0,
            top_k: 0,
            stream: false,
            timeout_ms: 0,
        }),
        metadata: Default::default(),
    };

    let infer_resp = service
        .infer(Request::new(infer_req))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(infer_resp.model, "fixture-model");
}

#[tokio::test]
async fn grpc_infer_stream() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bundles");
    let registry = std::sync::Arc::new(BundleModelRegistry::new(root));
    let service = grpc::RuntimeServiceImpl::new(registry.clone());

    let load_req = grpc::runtime::LoadModelRequest {
        spec: Some(grpc::runtime::ModelSpec {
            name: "fixture-model".to_string(),
            version: "v0".to_string(),
            format: "gguf".to_string(),
            backend: "llamacpp".to_string(),
            source_kind: "local_path".to_string(),
            source: "models/fixture.gguf".to_string(),
            sha256: "".to_string(),
            backend_settings_json: "{}".to_string(),
            labels: Default::default(),
        }),
    };

    let load_resp = service
        .load_model(Request::new(load_req))
        .await
        .unwrap()
        .into_inner();

    let infer_req = grpc::runtime::InferRequest {
        request_id: "req-2".to_string(),
        model_handle: load_resp.model_handle,
        input: Some(grpc::runtime::infer_request::Input::Chat(
            grpc::runtime::ChatInput { messages: vec![] },
        )),
        params: Some(grpc::runtime::InferenceParams {
            max_tokens: 0,
            temperature: 0.0,
            top_p: 0.0,
            top_k: 0,
            stream: true,
            timeout_ms: 0,
        }),
        metadata: Default::default(),
    };

    let mut stream = service
        .infer_stream(Request::new(infer_req))
        .await
        .unwrap()
        .into_inner();

    let first = stream.next().await.unwrap().unwrap();
    assert_eq!(first.request_id, "req-2");
}
