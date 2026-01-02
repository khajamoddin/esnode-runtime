FROM rust:1.77-slim AS build
WORKDIR /src
COPY Cargo.toml ./
COPY crates ./crates
RUN cargo build -p runtime-server --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /src/target/release/runtime-server /app/runtime-server
EXPOSE 9090
CMD ["/app/runtime-server"]
