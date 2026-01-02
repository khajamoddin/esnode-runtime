# ESNODE Runtime Charter (v0)

## Mission
ESNODE Runtime provides an early-stage execution layer for LLM inference that is:
- CPU-first and hardware-agnostic
- governable by default (policy + audit) in the target design
- scalable by default (containers/Kubernetes) in the target design
- observable by default (metrics/traces/logs) in the target design

## What ESNODE Runtime IS
1) **Execution**
   - Wraps one or more inference engines (v0 target: llama.cpp/GGUF)
   - Provides a stable "Engine API" for generation, streaming, and model lifecycle
   - Offers predictable resource controls (threads, memory, concurrency, timeouts)

2) **Governance (enforcement, not dashboards)**
   - Policies are evaluated before and during execution (target)
   - Supports multi-tenant controls: auth, quotas, rate limits, data-access rules (target)
   - Produces an auditable trail: inputs, outputs, sources, policy decisions (target)

3) **Scale**
   - Runs locally, on a single server, or in Kubernetes
   - Provides operational primitives: health, readiness, backpressure, rollouts
   - Supports horizontal scale via stateless gateway + replicated runtime nodes

4) **Observability**
   - Emits OpenTelemetry traces by default (target)
   - Exposes Prometheus metrics by default (target)
   - Produces structured logs (JSON) with redaction options (target)

## What ESNODE Runtime is NOT (non-goals)
- Not a workflow builder (use Flowise/Dify/Langflow as ecosystem clients)
- Not a full observability platform UI (integrate with Phoenix/Langfuse/Opik)
- Not a vector database (integrate via connectors)
- Not a "model training" framework

## Design principles
- **Stable contracts**: external API + internal canonical request schema remain stable
- **Contract-first core**: runtime-core owns all request/response and backend contracts
- **Pluggability**: engines, vector DBs, telemetry backends, and APIs are replaceable
- **Safe-by-default**: deny-by-default policies available; never silently bypass governance
- **Portable deployments**: same config works on laptop -> Docker -> Kubernetes
- **Minimal v0**: ship a useful, reliable core before expanding feature surface

## v0 target outcomes
- A user can run ESNODE Runtime locally with a GGUF model and call it using:
  - OpenAI-compatible chat completions API (streaming supported)
- A user can list/load models through bundle-backed ESNODE-native endpoints
- A user can enable basic governance:
  - API key auth + per-key rate limit + token budget
- A user can observe:
  - p50/p95 latency, tokens/sec, errors, concurrent sessions
  - OTEL traces across request -> policy -> engine
- Optional (v0.x): a lightweight Runtime Studio UI for load/list/infer/health.

## Future-facing boundary
ESNODE Runtime will remain the execution + governance + scale layer.
Other tools remain plugins/clients connected via open standards.
