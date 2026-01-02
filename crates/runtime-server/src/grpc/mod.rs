use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
use tonic::{Request, Response, Status};
use tokio_stream::{self as stream};

use crate::registry::BundleRegistry;

pub mod runtime {
    tonic::include_proto!("esnode.runtime.v1");
}

pub mod models {
    tonic::include_proto!("esnode.models.v1");
}

#[derive(Default)]
pub struct RuntimeServiceImpl {
    registry: Option<BundleRegistry>,
}

impl RuntimeServiceImpl {
    pub fn new(registry: BundleRegistry) -> Self {
        Self {
            registry: Some(registry),
        }
    }

    fn registry(&self) -> Result<BundleRegistry, Status> {
        self.registry
            .as_ref()
            .cloned()
            .ok_or_else(|| Status::failed_precondition("registry unavailable"))
    }
}

#[tonic::async_trait]
impl runtime::runtime_service_server::RuntimeService for RuntimeServiceImpl {
    async fn health(
        &self,
        _request: Request<runtime::HealthRequest>,
    ) -> Result<Response<runtime::HealthResponse>, Status> {
        Ok(Response::new(runtime::HealthResponse {
            status: "ok".to_string(),
            version: "0.1.0".to_string(),
        }))
    }

    async fn load_model(
        &self,
        _request: Request<runtime::LoadModelRequest>,
    ) -> Result<Response<runtime::LoadModelResponse>, Status> {
        let registry = self.registry()?;
        let req = _request.into_inner();
        let spec = req
            .spec
            .ok_or_else(|| Status::invalid_argument("missing spec"))?;
        let handle = registry
            .resolve_spec(&spec.name)
            .await
            .map(|s| s.name)
            .map_err(|_| Status::not_found("bundle not found"))?;
        Ok(Response::new(runtime::LoadModelResponse {
            model_handle: handle,
        }))
    }

    async fn unload_model(
        &self,
        request: Request<runtime::UnloadModelRequest>,
    ) -> Result<Response<runtime::UnloadModelResponse>, Status> {
        let registry = self.registry()?;
        let handle = request.into_inner().model_handle;
        let ok = registry.resolve_spec(&handle).await.is_ok();
        Ok(Response::new(runtime::UnloadModelResponse { ok }))
    }

    async fn list_models(
        &self,
        _request: Request<runtime::ListModelsRequest>,
    ) -> Result<Response<runtime::ListModelsResponse>, Status> {
        let registry = self.registry()?;
        let models = registry.list_specs().await.map_err(|e| Status::internal(e.to_string()))?;
        let models = models.into_iter().map(core_spec_to_runtime).collect();
        Ok(Response::new(runtime::ListModelsResponse { models }))
    }

    async fn infer(
        &self,
        request: Request<runtime::InferRequest>,
    ) -> Result<Response<runtime::InferResponse>, Status> {
        let registry = self.registry()?;
        let req = request.into_inner();
        let spec = registry
            .resolve_spec(&req.model_handle)
            .await
            .map_err(|_| Status::not_found("model handle not found"))?;

        let output = match req.input {
            Some(runtime::infer_request::Input::Chat(_)) => {
                runtime::infer_response::Output::Chat(runtime::ChatOutput {
                    message: Some(runtime::ChatMessage {
                        role: "assistant".to_string(),
                        content: "stub response".to_string(),
                        name: "".to_string(),
                    }),
                })
            }
            Some(runtime::infer_request::Input::Completion(_)) => {
                runtime::infer_response::Output::Completion(runtime::CompletionOutput {
                    text: "stub response".to_string(),
                })
            }
            None => {
                return Err(Status::invalid_argument("missing input"));
            }
        };

        Ok(Response::new(runtime::InferResponse {
            request_id: req.request_id,
            model: spec.name,
            output: Some(output),
            usage: Some(runtime::TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
            metadata: req.metadata,
        }))
    }

    type InferStreamStream =
        Pin<Box<dyn Stream<Item = Result<runtime::StreamChunk, Status>> + Send>>;

    async fn infer_stream(
        &self,
        request: Request<runtime::InferRequest>,
    ) -> Result<Response<Self::InferStreamStream>, Status> {
        let registry = self.registry()?;
        let req = request.into_inner();
        let spec = registry
            .resolve_spec(&req.model_handle)
            .await
            .map_err(|_| Status::not_found("model handle not found"))?;

        let chunks = stream_chunks(req.request_id, spec.name);
        Ok(Response::new(Box::pin(chunks)))
    }
}

pub fn service() -> runtime::runtime_service_server::RuntimeServiceServer<RuntimeServiceImpl> {
    runtime::runtime_service_server::RuntimeServiceServer::new(RuntimeServiceImpl::default())
}

pub fn service_with_registry(
    registry: BundleRegistry,
) -> runtime::runtime_service_server::RuntimeServiceServer<RuntimeServiceImpl> {
    runtime::runtime_service_server::RuntimeServiceServer::new(RuntimeServiceImpl::new(registry))
}

fn core_spec_to_runtime(spec: runtime_core::contract::model::ModelSpec) -> runtime::ModelSpec {
    let (source_kind, source, sha256) = match spec.source {
        runtime_core::contract::model::ModelSource::LocalPath { path } => {
            ("local_path".to_string(), path, String::new())
        }
        runtime_core::contract::model::ModelSource::Http { url, sha256 } => (
            "http".to_string(),
            url,
            sha256.unwrap_or_default(),
        ),
        runtime_core::contract::model::ModelSource::Registry { name, digest } => (
            "registry".to_string(),
            name,
            digest.unwrap_or_default(),
        ),
    };

    let backend_settings_json =
        serde_json::to_string(&spec.backend_settings).unwrap_or_else(|_| "{}".to_string());

    runtime::ModelSpec {
        name: spec.name,
        version: spec.version,
        format: format!("{:?}", spec.format).to_lowercase(),
        backend: spec.backend,
        source_kind,
        source,
        sha256,
        backend_settings_json,
        labels: spec.labels,
    }
}

fn stream_chunks(
    request_id: String,
    model: String,
) -> impl Stream<Item = Result<runtime::StreamChunk, Status>> {
    let start = runtime::StreamChunk {
        request_id: request_id.clone(),
        chunk: Some(runtime::stream_chunk::Chunk::Start(runtime::StreamStart {
            model,
            metadata: Default::default(),
        })),
    };
    let delta = runtime::StreamChunk {
        request_id: request_id.clone(),
        chunk: Some(runtime::stream_chunk::Chunk::Delta(runtime::StreamDelta {
            delta_text: "stub stream response".to_string(),
        })),
    };
    let end = runtime::StreamChunk {
        request_id,
        chunk: Some(runtime::stream_chunk::Chunk::End(runtime::StreamEnd {
            usage: Some(runtime::TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            }),
        })),
    };

    stream::iter(vec![Ok(start), Ok(delta), Ok(end)])
}
