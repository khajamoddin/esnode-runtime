# ESNODE Runtime Architecture (v0)

## Overview
ESNODE Runtime is organized as a small runtime stack plus optional UI:
- **runtime-core (Rust crate)**: contract traits + request/response types. No backend deps.
- **runtime-server (Rust binary)**: HTTP endpoints (OpenAI compat + ESNODE-native) and optional gRPC.
- **gateway (Go)**: OpenAI-compatible API and ESNODE-native gRPC forwarding.
- **runtime-studio (Vite UI)**: optional GUI for local ops and demos.

Backends (llamacpp/onnxrt/torch) are stub crates today; they depend on `runtime-core` and plug into
`runtime-server` via the backend registry.

## Request flow
OpenAI path (HTTP):
```text
Client
  |
  v
Gateway (/v1/chat/completions)
  |
  v
Runtime server (HTTP /v1/chat/completions)
  |
  v
Router -> backend -> model cache
  |
  v
SSE/JSON response
```

ESNODE-native path (gRPC, optional):
```text
Client or Runtime Studio
  |
  v
Gateway (/esnode/v1/*)
  |
  v
Runtime server (gRPC RuntimeService)
  |
  v
Router -> backend -> model cache
```

## Core contracts
- **runtime-core**: `InferenceBackend`, `ModelSpec`, `InferRequest`, `StreamChunk`.
- **runtime-proto**: gRPC `RuntimeService` for internal multi-process setups.
- **runtime-server**: Router + bundle registry + cache/batching utilities.

## Model bundles and registry
Models are described in bundles under `bundles/<name>/model-spec.yaml`. The bundle registry:
- resolves local paths relative to the bundle directory
- optionally verifies SHA-256 when provided
- supports list/load/resolve for ESNODE-native endpoints

## Observability hooks
`runtime-core` defines a minimal `TelemetrySink` for counters, histograms, and events. The server
uses this interface but does not yet wire full OTel/Prometheus exporters.

## Deployment topology (v0)
- Gateway on `:8080`
- Runtime server HTTP on `:9090`
- Runtime server gRPC on `:9091` (requires `proto-gen` feature)

## Runtime Studio (GUI)
Runtime Studio lives under `integrations/runtime-studio/`. It calls the gateway or runtime-server
over HTTP and exposes health, model list/load, and inference (streaming + non-streaming).

## Evolution: sidecar-first to FFI
**v0 (sidecar-first, planned)**
- Runtime server can call a backend via HTTP/IPC (e.g., llama.cpp sidecar).
- Pros: fast integration, fewer build/tooling constraints.
- Cons: extra hop, separate lifecycle.

**v0.x/v1 (FFI, planned)**
- Runtime server links backends via FFI or native bindings.
- Pros: lower latency, tighter resource control.
- Cons: build complexity, more platform constraints.

## Security and governance
- Gateway is the policy enforcement point (auth, RPM, concurrency, token budgets).
- Runtime server provides defense-in-depth (timeouts, cache limits).
