FROM golang:1.21-alpine AS build
WORKDIR /src
COPY go.mod ./
COPY cmd ./cmd
RUN go build -o /out/gateway ./cmd/gateway

FROM alpine:3.19
WORKDIR /app
COPY --from=build /out/gateway /app/gateway
EXPOSE 8080
CMD ["/app/gateway"]
