# ESNODE Runtime Architecture (v0)

## Overview
ESNODE Runtime is split into two primary services:
- **Gateway (Go)**: OpenAI-compatible API, policy enforcement, routing, telemetry.
- **Runtime node (Rust)**: execution engine wrapper, model lifecycle, streaming tokens.

v0 targets CPU-first GGUF inference via **llama.cpp**, initially as a sidecar process.

## Request flow (v0)
```text
Client
  |
  v
Gateway (OpenAI-compatible API)
  |
  |-- policy.PreExecute (auth, rpm, token budget)
  |
  v
Runtime node (ExecutionEngine)
  |
  v
llama.cpp sidecar (HTTP/IPC)
  |
  v
Runtime node (stream tokens)
  |
  v
Gateway (SSE stream + metrics/traces)
  |
  v
Client
```

## Core contracts
- **CanonicalRequest**: stable schema shared between gateway and runtime node.
- **ExecutionEngine**: Rust trait for model lifecycle, streaming generation, health.
- **PolicyEngine**: Go interface for pre/post execution enforcement.

## Observability
- **Metrics**: Prometheus counters/gauges/histograms for requests, errors, tokens, latency.
- **Tracing**: OpenTelemetry spans across gateway -> policy -> runtime -> engine.
- **Logs**: JSON logs with optional redaction flags from policy decisions.

## Deployment topology (v0)
- Single gateway + one or more runtime nodes.
- Runtime nodes are stateless per request; model loaded per node.
- Gateway performs basic routing (round-robin is acceptable in v0).

## Evolution: sidecar-first to FFI
**v0 (sidecar-first)**
- Runtime node uses llama.cpp as a subprocess or remote HTTP/IPC service.
- Pros: fast integration, fewer build/tooling constraints.
- Cons: extra hop, separate lifecycle.

**v0.x/v1 (FFI)**
- Runtime node links llama.cpp via FFI or native binding.
- Pros: lower latency, tighter resource control.
- Cons: build complexity, more platform constraints.

## Security and governance
- Gateway enforces API keys, RPM, concurrency, and token budgets.
- Runtime node applies hard limits as defense-in-depth (timeouts, max sessions, ctx size).
- Audit tags flow from policy decisions into telemetry and logs.
