# ESNODE Runtime Charter (v0)

## Mission
ESNODE Runtime provides a production-grade execution layer for LLM inference that is:
- CPU-first and hardware-agnostic
- governable by default (policy + audit)
- scalable by default (containers/Kubernetes)
- observable by default (metrics/traces/logs)

## What ESNODE Runtime IS
1) **Execution**
   - Wraps one or more inference engines (v0: llama.cpp/GGUF)
   - Provides a stable "Engine API" for generation, streaming, and model lifecycle
   - Offers predictable resource controls (threads, memory, concurrency, timeouts)

2) **Governance (enforcement, not dashboards)**
   - Policies are evaluated before and during execution
   - Supports multi-tenant controls: auth, quotas, rate limits, data-access rules
   - Produces an auditable trail: inputs, outputs, sources, policy decisions

3) **Scale**
   - Runs locally, on a single server, or in Kubernetes
   - Provides operational primitives: health, readiness, backpressure, rollouts
   - Supports horizontal scale via stateless gateway + replicated runtime nodes

4) **Observability**
   - Emits OpenTelemetry traces by default
   - Exposes Prometheus metrics by default
   - Produces structured logs (JSON) with redaction options

## What ESNODE Runtime is NOT (non-goals)
- Not a workflow builder (use Flowise/Dify/Langflow as ecosystem clients)
- Not a full observability platform UI (integrate with Phoenix/Langfuse/Opik)
- Not a vector database (integrate via connectors)
- Not a "model training" framework

## Design principles
- **Stable contracts**: external API + internal canonical request schema remain stable
- **Pluggability**: engines, vector DBs, telemetry backends, and APIs are replaceable
- **Safe-by-default**: deny-by-default policies available; never silently bypass governance
- **Portable deployments**: same config works on laptop -> Docker -> Kubernetes
- **Minimal v0**: ship a useful, reliable core before expanding feature surface

## v0 target outcomes
- A user can run ESNODE Runtime locally with a GGUF model and call it using:
  - OpenAI-compatible chat completions API (streaming supported)
- A user can enable basic governance:
  - API key auth + per-key rate limit + token budget
- A user can observe:
  - p50/p95 latency, tokens/sec, errors, concurrent sessions
  - OTEL traces across request -> policy -> engine

## Future-facing boundary
ESNODE Runtime will remain the execution + governance + scale layer.
Other tools remain plugins/clients connected via open standards.
