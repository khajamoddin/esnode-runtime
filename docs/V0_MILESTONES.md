# ESNODE Runtime v0 Milestones

## Definition of Done (v0)
A user can:
1) Run ESNODE Runtime locally with a GGUF model
2) Call it via an OpenAI-compatible Chat Completions API (streaming)
3) Enable basic governance (API keys + rate limit + token budget)
4) Get basic telemetry (Prometheus metrics + OTEL traces)

---

## M0 — Repo + build plumbing
- [ ] Monorepo created with runtime-node (Rust) + gateway (Go)
- [ ] Dockerfiles for gateway and runtime-node
- [ ] docker-compose.yaml that runs:
  - gateway
  - runtime-node (and llama.cpp sidecar if used)
- [ ] CI: build + unit tests

## M1 — Runtime-node (Rust) + llama.cpp execution
- [ ] Implement ExecutionEngine using llama.cpp sidecar OR FFI
- [ ] Load model from a configured path
- [ ] Generate streaming tokens
- [ ] /health, /ready endpoints
- [ ] Hard limits: request timeout, max concurrent sessions, max ctx length

## M2 — Gateway (Go) APIs
- [ ] OpenAI-compatible endpoint:
  - POST /v1/chat/completions
  - streaming (SSE)
- [ ] API key auth middleware
- [ ] Basic routing to runtime nodes (single node is OK in v0)
- [ ] Request id propagation end-to-end

## M3 — Governance (enforcement)
- [ ] PolicyEngine interface wired
- [ ] Policy pack: RPM limit
- [ ] Policy pack: concurrency cap
- [ ] Policy pack: token budget cap
- [ ] Audit log entry per request (decision + tags)

## M4 — Telemetry (minimum viable)
- [ ] Prometheus metrics:
  - requests_total, errors_total
  - request_latency_ms histogram
  - tokens_generated_total
  - concurrent_sessions gauge
- [ ] OpenTelemetry traces:
  - gateway span -> policy span -> engine span
- [ ] Structured logs with redaction option

## M5 — "Hello production"
- [ ] Helm chart scaffold (single namespace deploy)
- [ ] K8s manifests for:
  - gateway Deployment + Service
  - runtime-node Deployment + Service
- [ ] Readiness/liveness probes working
- [ ] Simple horizontal scaling:
  - N runtime-node replicas behind a Service
  - gateway load balances per request

---

## Stretch goals (v0.1)
- [ ] Add pgvector connector (RAG minimal)
- [ ] Add a minimal GUI page for configuring:
  - model path
  - limits
  - telemetry endpoints
- [ ] Basic eval runner (latency + "groundedness" smoke test)
