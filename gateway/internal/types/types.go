package types

type CanonicalMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

type CanonicalRequest struct {
	RequestID string             `json:"request_id"`
	TenantID  string             `json:"tenant_id"`
	ModelID   string             `json:"model_id"`
	Messages  []CanonicalMessage `json:"messages"`
	MaxTokens uint32             `json:"max_tokens"`
	Stream    bool               `json:"stream"`
}
