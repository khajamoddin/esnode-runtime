package main

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"os"
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

func runtimeNodeURL() string {
	if v := os.Getenv("RUNTIME_NODE_URL"); v != "" {
		return v
	}
	return "http://localhost:9090"
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
		runtimeNodeURL()+"/v1/chat/completions",
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

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/v1/chat/completions", chatCompletionsHandler)
	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(http.StatusOK) })

	addr := ":8080"
	log.Printf("gateway listening on %s", addr)
	if err := http.ListenAndServe(addr, mux); err != nil {
		log.Fatalf("server error: %v", err)
	}
}
