// Code generated manually for minimal gRPC client use. DO NOT EDIT.
package runtimev1

import (
	context "context"
	proto "github.com/golang/protobuf/proto"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

type HealthRequest struct{}

func (m *HealthRequest) Reset()         { *m = HealthRequest{} }
func (m *HealthRequest) String() string { return proto.CompactTextString(m) }
func (*HealthRequest) ProtoMessage()    {}

type HealthResponse struct {
	Status  string `protobuf:"bytes,1,opt,name=status,proto3" json:"status,omitempty"`
	Version string `protobuf:"bytes,2,opt,name=version,proto3" json:"version,omitempty"`
}

func (m *HealthResponse) Reset()         { *m = HealthResponse{} }
func (m *HealthResponse) String() string { return proto.CompactTextString(m) }
func (*HealthResponse) ProtoMessage()    {}

type ModelSpec struct {
	Name                string            `protobuf:"bytes,1,opt,name=name,proto3" json:"name,omitempty"`
	Version             string            `protobuf:"bytes,2,opt,name=version,proto3" json:"version,omitempty"`
	Format              string            `protobuf:"bytes,3,opt,name=format,proto3" json:"format,omitempty"`
	Backend             string            `protobuf:"bytes,4,opt,name=backend,proto3" json:"backend,omitempty"`
	SourceKind          string            `protobuf:"bytes,5,opt,name=source_kind,json=sourceKind,proto3" json:"source_kind,omitempty"`
	Source              string            `protobuf:"bytes,6,opt,name=source,proto3" json:"source,omitempty"`
	Sha256              string            `protobuf:"bytes,7,opt,name=sha256,proto3" json:"sha256,omitempty"`
	BackendSettingsJson string            `protobuf:"bytes,8,opt,name=backend_settings_json,json=backendSettingsJson,proto3" json:"backend_settings_json,omitempty"`
	Labels              map[string]string `protobuf:"bytes,9,rep,name=labels,proto3" json:"labels,omitempty" protobuf_key:"bytes,1,opt,name=key,proto3" protobuf_val:"bytes,2,opt,name=value,proto3"`
}

func (m *ModelSpec) Reset()         { *m = ModelSpec{} }
func (m *ModelSpec) String() string { return proto.CompactTextString(m) }
func (*ModelSpec) ProtoMessage()    {}

type LoadModelRequest struct {
	Spec *ModelSpec `protobuf:"bytes,1,opt,name=spec,proto3" json:"spec,omitempty"`
}

func (m *LoadModelRequest) Reset()         { *m = LoadModelRequest{} }
func (m *LoadModelRequest) String() string { return proto.CompactTextString(m) }
func (*LoadModelRequest) ProtoMessage()    {}

type LoadModelResponse struct {
	ModelHandle string `protobuf:"bytes,1,opt,name=model_handle,json=modelHandle,proto3" json:"model_handle,omitempty"`
}

func (m *LoadModelResponse) Reset()         { *m = LoadModelResponse{} }
func (m *LoadModelResponse) String() string { return proto.CompactTextString(m) }
func (*LoadModelResponse) ProtoMessage()    {}

type UnloadModelRequest struct {
	ModelHandle string `protobuf:"bytes,1,opt,name=model_handle,json=modelHandle,proto3" json:"model_handle,omitempty"`
}

func (m *UnloadModelRequest) Reset()         { *m = UnloadModelRequest{} }
func (m *UnloadModelRequest) String() string { return proto.CompactTextString(m) }
func (*UnloadModelRequest) ProtoMessage()    {}

type UnloadModelResponse struct {
	Ok bool `protobuf:"varint,1,opt,name=ok,proto3" json:"ok,omitempty"`
}

func (m *UnloadModelResponse) Reset()         { *m = UnloadModelResponse{} }
func (m *UnloadModelResponse) String() string { return proto.CompactTextString(m) }
func (*UnloadModelResponse) ProtoMessage()    {}

type ListModelsRequest struct{}

func (m *ListModelsRequest) Reset()         { *m = ListModelsRequest{} }
func (m *ListModelsRequest) String() string { return proto.CompactTextString(m) }
func (*ListModelsRequest) ProtoMessage()    {}

type ListModelsResponse struct {
	Models []*ModelSpec `protobuf:"bytes,1,rep,name=models,proto3" json:"models,omitempty"`
}

func (m *ListModelsResponse) Reset()         { *m = ListModelsResponse{} }
func (m *ListModelsResponse) String() string { return proto.CompactTextString(m) }
func (*ListModelsResponse) ProtoMessage()    {}

type ChatMessage struct {
	Role    string `protobuf:"bytes,1,opt,name=role,proto3" json:"role,omitempty"`
	Content string `protobuf:"bytes,2,opt,name=content,proto3" json:"content,omitempty"`
	Name    string `protobuf:"bytes,3,opt,name=name,proto3" json:"name,omitempty"`
}

func (m *ChatMessage) Reset()         { *m = ChatMessage{} }
func (m *ChatMessage) String() string { return proto.CompactTextString(m) }
func (*ChatMessage) ProtoMessage()    {}

type InferenceParams struct {
	MaxTokens   uint32  `protobuf:"varint,1,opt,name=max_tokens,json=maxTokens,proto3" json:"max_tokens,omitempty"`
	Temperature float32 `protobuf:"fixed32,2,opt,name=temperature,proto3" json:"temperature,omitempty"`
	TopP        float32 `protobuf:"fixed32,3,opt,name=top_p,json=topP,proto3" json:"top_p,omitempty"`
	TopK        uint32  `protobuf:"varint,4,opt,name=top_k,json=topK,proto3" json:"top_k,omitempty"`
	Stream      bool    `protobuf:"varint,5,opt,name=stream,proto3" json:"stream,omitempty"`
	TimeoutMs   uint64  `protobuf:"varint,6,opt,name=timeout_ms,json=timeoutMs,proto3" json:"timeout_ms,omitempty"`
}

func (m *InferenceParams) Reset()         { *m = InferenceParams{} }
func (m *InferenceParams) String() string { return proto.CompactTextString(m) }
func (*InferenceParams) ProtoMessage()    {}

type InferRequest struct {
	RequestId   string               `protobuf:"bytes,1,opt,name=request_id,json=requestId,proto3" json:"request_id,omitempty"`
	ModelHandle string               `protobuf:"bytes,2,opt,name=model_handle,json=modelHandle,proto3" json:"model_handle,omitempty"`
	Input       isInferRequest_Input `protobuf_oneof:"input"`
	Params      *InferenceParams     `protobuf:"bytes,5,opt,name=params,proto3" json:"params,omitempty"`
	Metadata    map[string]string    `protobuf:"bytes,6,rep,name=metadata,proto3" json:"metadata,omitempty" protobuf_key:"bytes,1,opt,name=key,proto3" protobuf_val:"bytes,2,opt,name=value,proto3"`
}

func (m *InferRequest) Reset()         { *m = InferRequest{} }
func (m *InferRequest) String() string { return proto.CompactTextString(m) }
func (*InferRequest) ProtoMessage()    {}

type isInferRequest_Input interface{ isInferRequest_Input() }

type InferRequest_Chat struct {
	Chat *ChatInput `protobuf:"bytes,3,opt,name=chat,proto3,oneof" json:"chat,omitempty"`
}

type InferRequest_Completion struct {
	Completion *CompletionInput `protobuf:"bytes,4,opt,name=completion,proto3,oneof" json:"completion,omitempty"`
}

func (*InferRequest_Chat) isInferRequest_Input() {}
func (*InferRequest_Completion) isInferRequest_Input() {}

func (*InferRequest) XXX_OneofWrappers() []interface{} {
	return []interface{}{
		(*InferRequest_Chat)(nil),
		(*InferRequest_Completion)(nil),
	}
}

type ChatInput struct {
	Messages []*ChatMessage `protobuf:"bytes,1,rep,name=messages,proto3" json:"messages,omitempty"`
}

func (m *ChatInput) Reset()         { *m = ChatInput{} }
func (m *ChatInput) String() string { return proto.CompactTextString(m) }
func (*ChatInput) ProtoMessage()    {}

type CompletionInput struct {
	Prompt string `protobuf:"bytes,1,opt,name=prompt,proto3" json:"prompt,omitempty"`
}

func (m *CompletionInput) Reset()         { *m = CompletionInput{} }
func (m *CompletionInput) String() string { return proto.CompactTextString(m) }
func (*CompletionInput) ProtoMessage()    {}

type TokenUsage struct {
	PromptTokens     uint32 `protobuf:"varint,1,opt,name=prompt_tokens,json=promptTokens,proto3" json:"prompt_tokens,omitempty"`
	CompletionTokens uint32 `protobuf:"varint,2,opt,name=completion_tokens,json=completionTokens,proto3" json:"completion_tokens,omitempty"`
	TotalTokens      uint32 `protobuf:"varint,3,opt,name=total_tokens,json=totalTokens,proto3" json:"total_tokens,omitempty"`
}

func (m *TokenUsage) Reset()         { *m = TokenUsage{} }
func (m *TokenUsage) String() string { return proto.CompactTextString(m) }
func (*TokenUsage) ProtoMessage()    {}

type InferResponse struct {
	RequestId string                `protobuf:"bytes,1,opt,name=request_id,json=requestId,proto3" json:"request_id,omitempty"`
	Model     string                `protobuf:"bytes,2,opt,name=model,proto3" json:"model,omitempty"`
	Output    isInferResponse_Output `protobuf_oneof:"output"`
	Usage     *TokenUsage           `protobuf:"bytes,5,opt,name=usage,proto3" json:"usage,omitempty"`
	Metadata  map[string]string     `protobuf:"bytes,6,rep,name=metadata,proto3" json:"metadata,omitempty" protobuf_key:"bytes,1,opt,name=key,proto3" protobuf_val:"bytes,2,opt,name=value,proto3"`
}

func (m *InferResponse) Reset()         { *m = InferResponse{} }
func (m *InferResponse) String() string { return proto.CompactTextString(m) }
func (*InferResponse) ProtoMessage()    {}

type isInferResponse_Output interface{ isInferResponse_Output() }

type InferResponse_Chat struct {
	Chat *ChatOutput `protobuf:"bytes,3,opt,name=chat,proto3,oneof" json:"chat,omitempty"`
}

type InferResponse_Completion struct {
	Completion *CompletionOutput `protobuf:"bytes,4,opt,name=completion,proto3,oneof" json:"completion,omitempty"`
}

func (*InferResponse_Chat) isInferResponse_Output() {}
func (*InferResponse_Completion) isInferResponse_Output() {}

func (*InferResponse) XXX_OneofWrappers() []interface{} {
	return []interface{}{
		(*InferResponse_Chat)(nil),
		(*InferResponse_Completion)(nil),
	}
}

type ChatOutput struct {
	Message *ChatMessage `protobuf:"bytes,1,opt,name=message,proto3" json:"message,omitempty"`
}

func (m *ChatOutput) Reset()         { *m = ChatOutput{} }
func (m *ChatOutput) String() string { return proto.CompactTextString(m) }
func (*ChatOutput) ProtoMessage()    {}

type CompletionOutput struct {
	Text string `protobuf:"bytes,1,opt,name=text,proto3" json:"text,omitempty"`
}

func (m *CompletionOutput) Reset()         { *m = CompletionOutput{} }
func (m *CompletionOutput) String() string { return proto.CompactTextString(m) }
func (*CompletionOutput) ProtoMessage()    {}

type StreamChunk struct {
	RequestId string              `protobuf:"bytes,1,opt,name=request_id,json=requestId,proto3" json:"request_id,omitempty"`
	Chunk     isStreamChunk_Chunk `protobuf_oneof:"chunk"`
}

func (m *StreamChunk) Reset()         { *m = StreamChunk{} }
func (m *StreamChunk) String() string { return proto.CompactTextString(m) }
func (*StreamChunk) ProtoMessage()    {}

type isStreamChunk_Chunk interface{ isStreamChunk_Chunk() }

type StreamChunk_Start struct {
	Start *StreamStart `protobuf:"bytes,2,opt,name=start,proto3,oneof" json:"start,omitempty"`
}

type StreamChunk_Delta struct {
	Delta *StreamDelta `protobuf:"bytes,3,opt,name=delta,proto3,oneof" json:"delta,omitempty"`
}

type StreamChunk_Event struct {
	Event *StreamEvent `protobuf:"bytes,4,opt,name=event,proto3,oneof" json:"event,omitempty"`
}

type StreamChunk_End struct {
	End *StreamEnd `protobuf:"bytes,5,opt,name=end,proto3,oneof" json:"end,omitempty"`
}

func (*StreamChunk_Start) isStreamChunk_Chunk() {}
func (*StreamChunk_Delta) isStreamChunk_Chunk() {}
func (*StreamChunk_Event) isStreamChunk_Chunk() {}
func (*StreamChunk_End) isStreamChunk_Chunk() {}

func (*StreamChunk) XXX_OneofWrappers() []interface{} {
	return []interface{}{
		(*StreamChunk_Start)(nil),
		(*StreamChunk_Delta)(nil),
		(*StreamChunk_Event)(nil),
		(*StreamChunk_End)(nil),
	}
}

type StreamStart struct {
	Model    string            `protobuf:"bytes,1,opt,name=model,proto3" json:"model,omitempty"`
	Metadata map[string]string `protobuf:"bytes,2,rep,name=metadata,proto3" json:"metadata,omitempty" protobuf_key:"bytes,1,opt,name=key,proto3" protobuf_val:"bytes,2,opt,name=value,proto3"`
}

func (m *StreamStart) Reset()         { *m = StreamStart{} }
func (m *StreamStart) String() string { return proto.CompactTextString(m) }
func (*StreamStart) ProtoMessage()    {}

type StreamDelta struct {
	DeltaText string `protobuf:"bytes,1,opt,name=delta_text,json=deltaText,proto3" json:"delta_text,omitempty"`
}

func (m *StreamDelta) Reset()         { *m = StreamDelta{} }
func (m *StreamDelta) String() string { return proto.CompactTextString(m) }
func (*StreamDelta) ProtoMessage()    {}

type StreamEvent struct {
	Name     string `protobuf:"bytes,1,opt,name=name,proto3" json:"name,omitempty"`
	DataJson string `protobuf:"bytes,2,opt,name=data_json,json=dataJson,proto3" json:"data_json,omitempty"`
}

func (m *StreamEvent) Reset()         { *m = StreamEvent{} }
func (m *StreamEvent) String() string { return proto.CompactTextString(m) }
func (*StreamEvent) ProtoMessage()    {}

type StreamEnd struct {
	Usage *TokenUsage `protobuf:"bytes,1,opt,name=usage,proto3" json:"usage,omitempty"`
}

func (m *StreamEnd) Reset()         { *m = StreamEnd{} }
func (m *StreamEnd) String() string { return proto.CompactTextString(m) }
func (*StreamEnd) ProtoMessage()    {}

type RuntimeServiceClient interface {
	Health(ctx context.Context, in *HealthRequest, opts ...grpc.CallOption) (*HealthResponse, error)
	LoadModel(ctx context.Context, in *LoadModelRequest, opts ...grpc.CallOption) (*LoadModelResponse, error)
	UnloadModel(ctx context.Context, in *UnloadModelRequest, opts ...grpc.CallOption) (*UnloadModelResponse, error)
	ListModels(ctx context.Context, in *ListModelsRequest, opts ...grpc.CallOption) (*ListModelsResponse, error)
	Infer(ctx context.Context, in *InferRequest, opts ...grpc.CallOption) (*InferResponse, error)
	InferStream(ctx context.Context, in *InferRequest, opts ...grpc.CallOption) (RuntimeService_InferStreamClient, error)
}

type runtimeServiceClient struct{ cc *grpc.ClientConn }

func NewRuntimeServiceClient(cc *grpc.ClientConn) RuntimeServiceClient {
	return &runtimeServiceClient{cc}
}

func (c *runtimeServiceClient) Health(ctx context.Context, in *HealthRequest, opts ...grpc.CallOption) (*HealthResponse, error) {
	out := new(HealthResponse)
	err := c.cc.Invoke(ctx, "/esnode.runtime.v1.RuntimeService/Health", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *runtimeServiceClient) LoadModel(ctx context.Context, in *LoadModelRequest, opts ...grpc.CallOption) (*LoadModelResponse, error) {
	out := new(LoadModelResponse)
	err := c.cc.Invoke(ctx, "/esnode.runtime.v1.RuntimeService/LoadModel", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *runtimeServiceClient) UnloadModel(ctx context.Context, in *UnloadModelRequest, opts ...grpc.CallOption) (*UnloadModelResponse, error) {
	out := new(UnloadModelResponse)
	err := c.cc.Invoke(ctx, "/esnode.runtime.v1.RuntimeService/UnloadModel", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *runtimeServiceClient) ListModels(ctx context.Context, in *ListModelsRequest, opts ...grpc.CallOption) (*ListModelsResponse, error) {
	out := new(ListModelsResponse)
	err := c.cc.Invoke(ctx, "/esnode.runtime.v1.RuntimeService/ListModels", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *runtimeServiceClient) Infer(ctx context.Context, in *InferRequest, opts ...grpc.CallOption) (*InferResponse, error) {
	out := new(InferResponse)
	err := c.cc.Invoke(ctx, "/esnode.runtime.v1.RuntimeService/Infer", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *runtimeServiceClient) InferStream(ctx context.Context, in *InferRequest, opts ...grpc.CallOption) (RuntimeService_InferStreamClient, error) {
	stream, err := c.cc.NewStream(ctx, &_RuntimeService_serviceDesc.Streams[0], "/esnode.runtime.v1.RuntimeService/InferStream", opts...)
	if err != nil {
		return nil, err
	}
	x := &runtimeServiceInferStreamClient{stream}
	if err := x.ClientStream.SendMsg(in); err != nil {
		return nil, err
	}
	if err := x.ClientStream.CloseSend(); err != nil {
		return nil, err
	}
	return x, nil
}

type RuntimeService_InferStreamClient interface {
	Recv() (*StreamChunk, error)
	grpc.ClientStream
}

type runtimeServiceInferStreamClient struct{ grpc.ClientStream }

func (x *runtimeServiceInferStreamClient) Recv() (*StreamChunk, error) {
	m := new(StreamChunk)
	if err := x.ClientStream.RecvMsg(m); err != nil {
		return nil, err
	}
	return m, nil
}

var _RuntimeService_serviceDesc = grpc.ServiceDesc{
	ServiceName: "esnode.runtime.v1.RuntimeService",
	HandlerType: (*interface{})(nil),
	Streams: []grpc.StreamDesc{
		{
			StreamName:    "InferStream",
			ServerStreams: true,
		},
	},
}

func init() {
	_ = codes.OK
	_ = status.New
}
