package policy

import "time"

type Decision string

const (
	DecisionAllow Decision = "ALLOW"
	DecisionDeny  Decision = "DENY"
)

type PolicyContext struct {
	RequestID string
	TenantID  string
	APIKeyID  string
	IP        string
	UserAgent string
	Now       time.Time
}

type CanonicalMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type CanonicalRequest struct {
	RequestID string             `json:"request_id"`
	TenantID  string             `json:"tenant_id"`
	ModelID   string             `json:"model_id"`
	Messages  []CanonicalMessage `json:"messages"`
	// Governance-relevant hints:
	MaxTokens uint32 `json:"max_tokens"`
	Stream    bool   `json:"stream"`
}

type PolicyDecision struct {
	Decision Decision `json:"decision"`
	Reason   string   `json:"reason,omitempty"`

	// Enforcement outputs (defense-in-depth)
	// e.g. rate limit / quotas / budgets / redaction flags
	TokenBudget uint32 `json:"token_budget,omitempty"`
	RPM         uint32 `json:"rpm,omitempty"`
	Concurrency uint32 `json:"concurrency,omitempty"`

	// Audit tags to emit into telemetry/logging
	Tags map[string]string `json:"tags,omitempty"`
}

type PolicyEngine interface {
	// Evaluate before sending to runtime node.
	PreExecute(ctx PolicyContext, req CanonicalRequest) (PolicyDecision, error)

	// Optional: evaluate after response for audit/alerts (v0 can be no-op).
	PostExecute(ctx PolicyContext, req CanonicalRequest, outcome ExecutionOutcome) error
}

type ExecutionOutcome struct {
	Status       string `json:"status"` // "ok" | "error" | "timeout"
	LatencyMs    uint64 `json:"latency_ms"`
	TotalTokens  uint32 `json:"total_tokens"`
	ErrorCode    string `json:"error_code,omitempty"`
	ErrorMessage string `json:"error_message,omitempty"`
}
