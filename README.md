# ESNODE Runtime

**ESNODE Runtime** is a production-grade LLM **execution + governance + scale** layer.

It wraps fast, portable inference engines (v0: **llama.cpp / GGUF**) and adds what teams actually need to run LLMs in real environments:
- **Governance** (policy enforcement, quotas, audit)
- **Scale** (Docker/Kubernetes-ready, multi-node)
- **Observability** (OpenTelemetry + Prometheus)

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

## Core capabilities (v0)

### Execution
- GGUF model serving via llama.cpp (CPU-first)
- Streaming generation
- Concurrency and time budgets (defense-in-depth)

### Governance (enforced)
- API key authentication
- Rate limits (RPM)
- Concurrency limits
- Token budgets
- Audit logs

### Scale
- Container-first deployment
- Kubernetes-ready (deploy multiple runtime nodes)
- Gateway routes traffic across nodes

### Observability
- Prometheus metrics (latency, tokens/sec, errors)
- OpenTelemetry traces (gateway -> policy -> engine)

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
    "model": "local-gguf",
    "stream": true,
    "messages": [{"role":"user","content":"Hello from ESNODE Runtime"}]
  }'
```

---

## Repository layout
- runtime-node/ — Rust runtime node that wraps the inference engine
- gateway/ — Go gateway exposing OpenAI-compatible API + governance
- docs/ — charter, milestones, architecture
- deployments/ — docker-compose and Kubernetes/Helm scaffolding

---

## Roadmap (high-level)
- v0.1: pluggable VectorDB connectors (pgvector/Qdrant) and minimal RAG
- v0.2: GUI configuration ("Runtime Studio") for profiles, telemetry, governance
- v0.3: Kubernetes Operator (CRDs), eval gates, rollout policies

---

## License
TBD (project license will be defined after repo initialization and later stage of the project).
Third-party engines/tools remain under their respective licenses.
