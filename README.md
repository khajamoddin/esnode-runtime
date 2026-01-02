# ESNODE Runtime

**ESNODE Runtime** is an early-stage LLM **execution + governance + scale** layer with
scaffolding in place for backends, policy hooks, and observability.

The current repo ships a stub runtime server + gateway, plus a Vite-based Runtime Studio UI.
Real backends (v0 target: **llama.cpp / GGUF**) and enforcement/telemetry are planned but not yet
implemented.

> ESNODE Runtime is not a workflow builder UI.
> It is the runtime layer that existing tools can plug into.

---

## Why ESNODE Runtime exists

Most open-source LLM stacks are either:
- inference engines, or
- app builders, or
- observability dashboards

Teams still struggle with:
- safe multi-tenant serving
- predictable resource controls on CPU fleets
- deployment gating, policy enforcement, audit trails
- repeatable ops on Kubernetes

ESNODE Runtime focuses on the missing "runtime" layer.

---

## Core capabilities (current state)

### Execution
- HTTP + optional gRPC runtime server with stub inference backend
- Streaming response plumbing (SSE / gRPC stream)
- Model bundle registry with basic validation
- Router + in-memory model cache

### Governance (enforced)
- Policy interfaces are defined
- Enforcement and rate limits are not wired yet

### Scale
- Container-first Dockerfiles + Compose scaffolding
- Kubernetes scaffolding directory (empty)

### Observability
- Telemetry interface exists
- No Prometheus/OTel exporters wired yet

---

## Compatible ecosystem clients (examples)
Because ESNODE Runtime exposes an OpenAI-compatible API, it can be used with existing UIs and builders (deployed separately), such as Open WebUI, Dify, Flowise, Langflow, etc.

---

## Quick start (local)

### 1) Requirements
- Docker + Docker Compose
- A GGUF model file available locally

### 2) Run
```bash
docker compose up --build
```

### 3) Call the OpenAI-compatible endpoint
```bash
curl http://localhost:8080/v1/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "fixture-model",
    "stream": false,
    "messages": [{"role":"user","content":"Hello from ESNODE Runtime"}]
  }'
```

### 4) ESNODE-native endpoints (bundle-backed)
```bash
curl http://localhost:8080/esnode/v1/models
```

---

## Model bundles
Bundles live under `bundles/<name>/model-spec.yaml`. The runtime-server resolves relative paths
inside the bundle directory and can verify SHA-256 checksums when provided.

Example layout:
```text
bundles/
  fixture-model/
    model-spec.yaml
    models/
      fixture.gguf
```

---

## Runtime Studio (GUI)
Runtime Studio is a Vite-based UI for operating the runtime.

```bash
cd integrations/runtime-studio
npm install
npm run dev
```

Default base URL is `http://localhost:8080` (gateway). Use `http://localhost:9090` to hit the
runtime-server directly. The GUI supports:
- health/ready checks
- list/load bundle models
- ESNODE inference (streaming + non-streaming)
- metrics link

---

## gRPC (optional)
gRPC builds are gated behind the `proto-gen` feature to avoid requiring `protoc` for default builds.

```bash
cargo test -p runtime-server --features proto-gen
```

To run runtime-server with gRPC enabled:
```bash
cargo run -p runtime-server --features proto-gen
```

---

## Repository layout
- crates/runtime-core/ — Rust contracts + types (backend-agnostic)
- crates/runtime-server/ — Rust HTTP server with optional gRPC and stub backend
- crates/runtime-backend/ — backend registry + loader
- crates/backend-*/ — backend implementations (onnxrt/llamacpp/torch)
- crates/runtime-cli/ — `esnode` CLI stub
- crates/runtime-proto/ — gRPC proto definitions
- gateway/ — Go gateway exposing OpenAI-compatible API + ESNODE-native gRPC forwarding (stub)
- api/ — OpenAPI + examples
- bundles/ — sample model bundles
- integrations/ — docker + kubernetes scaffolding
- integrations/runtime-studio/ — Runtime Studio (Vite GUI)
- docs/ — charter, milestones, architecture
- deployments/ — legacy docker-compose location

---

## Linting (v0 defaults)
- Rust: `cargo clippy --workspace --all-targets`
- Go: `gofmt` (already used on gateway code)

---

## Roadmap (high-level)
- v0.1: pluggable VectorDB connectors (pgvector/Qdrant) and minimal RAG
- v0.2: GUI configuration ("Runtime Studio") for profiles, telemetry, governance
- v0.3: Kubernetes Operator (CRDs), eval gates, rollout policies

---

## License
TBD (project license will be defined after repo initialization and later stage of the project).
Third-party engines/tools remain under their respective licenses.
