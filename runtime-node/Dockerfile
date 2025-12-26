FROM rust:1.77-slim AS build
WORKDIR /src
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /src/target/release/runtime-node /app/runtime-node
EXPOSE 9090
CMD ["/app/runtime-node"]
