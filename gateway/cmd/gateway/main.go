package main

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
	"time"

	"esnode-runtime/gateway/internal/runtimev1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type chatCompletionRequest struct {
	Model    string `json:"model"`
	Stream   bool   `json:"stream"`
	Messages []struct {
		Role    string `json:"role"`
		Content string `json:"content"`
	} `json:"messages"`
}

type chatCompletionResponse struct {
	ID      string `json:"id"`
	Object  string `json:"object"`
	Created int64  `json:"created"`
	Model   string `json:"model"`
	Choices []struct {
		Index   int `json:"index"`
		Message struct {
			Role    string `json:"role"`
			Content string `json:"content"`
		} `json:"message"`
		FinishReason string `json:"finish_reason"`
	} `json:"choices"`
}

func runtimeServerURL() string {
	if v := os.Getenv("RUNTIME_SERVER_URL"); v != "" {
		return v
	}
	if v := os.Getenv("RUNTIME_NODE_URL"); v != "" {
		return v
	}
	return "http://localhost:9090"
}

func runtimeServerGRPCAddr() string {
	if v := os.Getenv("RUNTIME_SERVER_GRPC_ADDR"); v != "" {
		return v
	}
	return "localhost:9091"
}

func proxyHealth(w http.ResponseWriter, r *http.Request, path string) {
	upstreamReq, err := http.NewRequestWithContext(
		r.Context(),
		http.MethodGet,
		runtimeServerURL()+path,
		nil,
	)
	if err != nil {
		http.Error(w, "upstream request error", http.StatusBadGateway)
		return
	}

	resp, err := http.DefaultClient.Do(upstreamReq)
	if err != nil {
		http.Error(w, "upstream unavailable", http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		http.Error(w, "upstream not ready", http.StatusServiceUnavailable)
		return
	}

	w.WriteHeader(http.StatusOK)
}

func chatCompletionsHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req chatCompletionRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}

	payload, err := json.Marshal(req)
	if err != nil {
		http.Error(w, "encode error", http.StatusInternalServerError)
		return
	}

	upstreamReq, err := http.NewRequestWithContext(
		r.Context(),
		http.MethodPost,
		runtimeServerURL()+"/v1/chat/completions",
		bytes.NewReader(payload),
	)
	if err != nil {
		http.Error(w, "upstream request error", http.StatusBadGateway)
		return
	}
	upstreamReq.Header.Set("Content-Type", "application/json")
	upstreamReq.Header.Set("Accept", "application/json")

	upstreamResp, err := http.DefaultClient.Do(upstreamReq)
	if err != nil {
		http.Error(w, "upstream unavailable", http.StatusBadGateway)
		return
	}
	defer upstreamResp.Body.Close()

	if req.Stream {
		for k, v := range upstreamResp.Header {
			for _, vv := range v {
				w.Header().Add(k, vv)
			}
		}
		w.WriteHeader(upstreamResp.StatusCode)

		buf := make([]byte, 4096)
		for {
			n, readErr := upstreamResp.Body.Read(buf)
			if n > 0 {
				if _, writeErr := w.Write(buf[:n]); writeErr != nil {
					return
				}
				if flusher, ok := w.(http.Flusher); ok {
					flusher.Flush()
				}
			}
			if readErr == io.EOF {
				return
			}
			if readErr != nil {
				return
			}
		}
	}

	if upstreamResp.StatusCode != http.StatusOK {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(upstreamResp.StatusCode)
		if _, err := io.Copy(w, upstreamResp.Body); err != nil {
			http.Error(w, "upstream read error", http.StatusBadGateway)
		}
		return
	}

	var upstream chatCompletionResponse
	if err := json.NewDecoder(upstreamResp.Body).Decode(&upstream); err != nil {
		http.Error(w, "invalid upstream json", http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	if err := json.NewEncoder(w).Encode(upstream); err != nil {
		http.Error(w, "encode error", http.StatusInternalServerError)
		return
	}
}

type esnodeModelSpec struct {
	Name            string            `json:"name"`
	Version         string            `json:"version"`
	Source          esnodeModelSource `json:"source"`
	Format          string            `json:"format"`
	Backend         string            `json:"backend"`
	BackendSettings map[string]any    `json:"backend_settings"`
	Labels          map[string]string `json:"labels"`
}

type esnodeModelSource struct {
	Kind         string `json:"kind"`
	Path         string `json:"path,omitempty"`
	URL          string `json:"url,omitempty"`
	Sha256       string `json:"sha256,omitempty"`
	RegistryName string `json:"registry_name,omitempty"`
	Digest       string `json:"digest,omitempty"`
}

type esnodeUnloadModelRequest struct {
	ModelHandle string `json:"model_handle"`
}

type esnodeInferRequest struct {
	RequestID string                 `json:"request_id"`
	Model     string                 `json:"model"`
	Input     esnodeInferInput       `json:"input"`
	Params    esnodeInferParams      `json:"params"`
	Metadata  map[string]string      `json:"metadata"`
	Extras    map[string]interface{} `json:"-"`
}

type esnodeInferInput struct {
	Type     string              `json:"type"`
	Messages []esnodeChatMessage `json:"messages,omitempty"`
	Prompt   string              `json:"prompt,omitempty"`
}

type esnodeChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
	Name    string `json:"name,omitempty"`
}

type esnodeInferParams struct {
	MaxTokens   uint32  `json:"max_tokens,omitempty"`
	Temperature float32 `json:"temperature,omitempty"`
	TopP        float32 `json:"top_p,omitempty"`
	TopK        uint32  `json:"top_k,omitempty"`
	Stream      bool    `json:"stream"`
	TimeoutMs   uint64  `json:"timeout_ms,omitempty"`
}

type esnodeInferResponse struct {
	RequestID string            `json:"request_id"`
	Model     string            `json:"model"`
	Output    esnodeOutput      `json:"output"`
	Usage     *esnodeUsage      `json:"usage,omitempty"`
	Metadata  map[string]string `json:"metadata"`
}

type esnodeOutput struct {
	Type    string             `json:"type"`
	Message *esnodeChatMessage `json:"message,omitempty"`
	Text    string             `json:"text,omitempty"`
}

type esnodeUsage struct {
	PromptTokens     uint32 `json:"prompt_tokens"`
	CompletionTokens uint32 `json:"completion_tokens"`
	TotalTokens      uint32 `json:"total_tokens"`
}

type streamChunkJSON struct {
	Kind      string            `json:"kind"`
	RequestID string            `json:"request_id"`
	Model     string            `json:"model,omitempty"`
	Metadata  map[string]string `json:"metadata,omitempty"`
	DeltaText string            `json:"delta_text,omitempty"`
	Name      string            `json:"name,omitempty"`
	Data      json.RawMessage   `json:"data,omitempty"`
	Usage     *esnodeUsage      `json:"usage,omitempty"`
}

func handleESNode(w http.ResponseWriter, r *http.Request) {
	ctx, cancel := context.WithTimeout(r.Context(), 15*time.Second)
	defer cancel()

	conn, err := grpc.DialContext(ctx, runtimeServerGRPCAddr(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		http.Error(w, "grpc connect error", http.StatusBadGateway)
		return
	}
	defer conn.Close()

	client := runtimev1.NewRuntimeServiceClient(conn)

	switch r.URL.Path {
	case "/esnode/v1/models/load":
		handleESNodeLoad(w, r, ctx, client)
	case "/esnode/v1/models/unload":
		handleESNodeUnload(w, r, ctx, client)
	case "/esnode/v1/models":
		handleESNodeList(w, r, ctx, client)
	case "/esnode/v1/infer":
		handleESNodeInfer(w, r, ctx, client)
	case "/esnode/v1/infer/stream":
		handleESNodeInferStream(w, r, ctx, client)
	default:
		http.Error(w, "not implemented", http.StatusNotImplemented)
	}
}

func handleESNodeLoad(w http.ResponseWriter, r *http.Request, ctx context.Context, client runtimev1.RuntimeServiceClient) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var spec esnodeModelSpec
	if err := json.NewDecoder(r.Body).Decode(&spec); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}

	sourceKind := spec.Source.Kind
	source := spec.Source.Path
	if sourceKind == "http" {
		source = spec.Source.URL
	} else if sourceKind == "registry" {
		source = spec.Source.RegistryName
	}

	backendSettingsJSON, err := json.Marshal(spec.BackendSettings)
	if err != nil {
		http.Error(w, "encode error", http.StatusBadRequest)
		return
	}

	resp, err := client.LoadModel(ctx, &runtimev1.LoadModelRequest{
		Spec: &runtimev1.ModelSpec{
			Name:                spec.Name,
			Version:             spec.Version,
			Format:              spec.Format,
			Backend:             spec.Backend,
			SourceKind:          sourceKind,
			Source:              source,
			Sha256:              spec.Source.Sha256,
			BackendSettingsJson: string(backendSettingsJSON),
			Labels:              spec.Labels,
		},
	})
	if err != nil {
		http.Error(w, "grpc error", http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]string{"model_handle": resp.ModelHandle})
}

func handleESNodeUnload(w http.ResponseWriter, r *http.Request, ctx context.Context, client runtimev1.RuntimeServiceClient) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req esnodeUnloadModelRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}

	resp, err := client.UnloadModel(ctx, &runtimev1.UnloadModelRequest{ModelHandle: req.ModelHandle})
	if err != nil {
		http.Error(w, "grpc error", http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]bool{"ok": resp.Ok})
}

func handleESNodeList(w http.ResponseWriter, r *http.Request, ctx context.Context, client runtimev1.RuntimeServiceClient) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	resp, err := client.ListModels(ctx, &runtimev1.ListModelsRequest{})
	if err != nil {
		http.Error(w, "grpc error", http.StatusBadGateway)
		return
	}

	models := make([]map[string]any, 0, len(resp.Models))
	for _, spec := range resp.Models {
		models = append(models, map[string]any{
			"name":             spec.Name,
			"version":          spec.Version,
			"format":           spec.Format,
			"backend":          spec.Backend,
			"backend_settings": json.RawMessage(spec.BackendSettingsJson),
			"labels":           spec.Labels,
		})
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(map[string]any{"models": models})
}

func handleESNodeInfer(w http.ResponseWriter, r *http.Request, ctx context.Context, client runtimev1.RuntimeServiceClient) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req esnodeInferRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}

	grpcReq, err := toGrpcInferRequest(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	resp, err := client.Infer(ctx, grpcReq)
	if err != nil {
		http.Error(w, "grpc error", http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	_ = json.NewEncoder(w).Encode(fromGrpcInferResponse(resp))
}

func handleESNodeInferStream(w http.ResponseWriter, r *http.Request, ctx context.Context, client runtimev1.RuntimeServiceClient) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}

	var req esnodeInferRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, "invalid json", http.StatusBadRequest)
		return
	}

	grpcReq, err := toGrpcInferRequest(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	grpcReq.Params.Stream = true

	stream, err := client.InferStream(ctx, grpcReq)
	if err != nil {
		http.Error(w, "grpc error", http.StatusBadGateway)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")

	flusher, _ := w.(http.Flusher)
	for {
		chunk, recvErr := stream.Recv()
		if recvErr == io.EOF {
			return
		}
		if recvErr != nil {
			return
		}
		payload, err := json.Marshal(fromGrpcStreamChunk(chunk))
		if err != nil {
			return
		}
		if _, err := w.Write([]byte("data: " + string(payload) + "\n\n")); err != nil {
			return
		}
		if flusher != nil {
			flusher.Flush()
		}
	}
}

func toGrpcInferRequest(req esnodeInferRequest) (*runtimev1.InferRequest, error) {
	grpcReq := &runtimev1.InferRequest{
		RequestId:   req.RequestID,
		ModelHandle: req.Model,
		Params: &runtimev1.InferenceParams{
			MaxTokens:   req.Params.MaxTokens,
			Temperature: req.Params.Temperature,
			TopP:        req.Params.TopP,
			TopK:        req.Params.TopK,
			Stream:      req.Params.Stream,
			TimeoutMs:   req.Params.TimeoutMs,
		},
		Metadata: req.Metadata,
	}

	switch req.Input.Type {
	case "chat":
		msgs := make([]*runtimev1.ChatMessage, 0, len(req.Input.Messages))
		for _, msg := range req.Input.Messages {
			msgs = append(msgs, &runtimev1.ChatMessage{
				Role:    msg.Role,
				Content: msg.Content,
				Name:    msg.Name,
			})
		}
		grpcReq.Input = &runtimev1.InferRequest_Chat{
			Chat: &runtimev1.ChatInput{Messages: msgs},
		}
	case "completion":
		grpcReq.Input = &runtimev1.InferRequest_Completion{
			Completion: &runtimev1.CompletionInput{Prompt: req.Input.Prompt},
		}
	default:
		return nil, http.ErrNotSupported
	}

	return grpcReq, nil
}

func fromGrpcInferResponse(resp *runtimev1.InferResponse) esnodeInferResponse {
	out := esnodeInferResponse{
		RequestID: resp.RequestId,
		Model:     resp.Model,
		Metadata:  resp.Metadata,
	}

	if resp.Usage != nil {
		out.Usage = &esnodeUsage{
			PromptTokens:     resp.Usage.PromptTokens,
			CompletionTokens: resp.Usage.CompletionTokens,
			TotalTokens:      resp.Usage.TotalTokens,
		}
	}

	switch v := resp.Output.(type) {
	case *runtimev1.InferResponse_Chat:
		msg := v.Chat.Message
		out.Output = esnodeOutput{
			Type: "chat",
			Message: &esnodeChatMessage{
				Role:    msg.Role,
				Content: msg.Content,
				Name:    msg.Name,
			},
		}
	case *runtimev1.InferResponse_Completion:
		out.Output = esnodeOutput{
			Type: "completion",
			Text: v.Completion.Text,
		}
	default:
		out.Output = esnodeOutput{Type: "completion", Text: ""}
	}

	return out
}

func fromGrpcStreamChunk(chunk *runtimev1.StreamChunk) streamChunkJSON {
	out := streamChunkJSON{
		RequestID: chunk.RequestId,
	}

	switch v := chunk.Chunk.(type) {
	case *runtimev1.StreamChunk_Start:
		out.Kind = "start"
		out.Model = v.Start.Model
		out.Metadata = v.Start.Metadata
	case *runtimev1.StreamChunk_Delta:
		out.Kind = "delta"
		out.DeltaText = v.Delta.DeltaText
	case *runtimev1.StreamChunk_Event:
		out.Kind = "event"
		out.Name = v.Event.Name
		if json.Valid([]byte(v.Event.DataJson)) {
			out.Data = json.RawMessage(v.Event.DataJson)
		} else {
			out.Data = json.RawMessage([]byte(`\"` + v.Event.DataJson + `\"`))
		}
	case *runtimev1.StreamChunk_End:
		out.Kind = "end"
		if v.End.Usage != nil {
			out.Usage = &esnodeUsage{
				PromptTokens:     v.End.Usage.PromptTokens,
				CompletionTokens: v.End.Usage.CompletionTokens,
				TotalTokens:      v.End.Usage.TotalTokens,
			}
		}
	default:
		out.Kind = "event"
	}

	return out
}

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/v1/chat/completions", chatCompletionsHandler)
	mux.HandleFunc("/esnode/v1/", handleESNode)
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		proxyHealth(w, r, "/healthz")
	})
	mux.HandleFunc("/readyz", func(w http.ResponseWriter, r *http.Request) {
		proxyHealth(w, r, "/readyz")
	})
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		proxyHealth(w, r, "/healthz")
	})

	addr := ":8080"
	log.Printf("gateway listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server error: %v", err)
	}
}
