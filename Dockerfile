# https://github.com/LukeMathWalker/cargo-chef/tree/v0.1.31#how-to-use
# https://github.com/LukeMathWalker/cargo-chef/tree/v0.1.31#pre-built-images
FROM lukemathwalker/cargo-chef:latest-rust-1.58.1-slim-buster@sha256:599bf734dc9332b41f853ab34abb84f2d335a048f68f40735fe1eeaf53a949e5 AS chef
# Cache-bust when this file changes
COPY rust-toolchain /
# See also: /rust-toolchain
RUN rustup toolchain install 1.58.1
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json \
    --features otel \
    --release
COPY . .
RUN rm -rf .cargo && \
    SQLX_OFFLINE=true \
    cargo build --features otel --release

FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/axum-rest-example /usr/local/bin
COPY config /app/config
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/axum-rest-example"]
