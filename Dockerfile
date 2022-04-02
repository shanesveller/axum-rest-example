# https://github.com/LukeMathWalker/cargo-chef/tree/v0.1.35#how-to-use
# https://github.com/LukeMathWalker/cargo-chef/tree/v0.1.35#pre-built-images
FROM lukemathwalker/cargo-chef:0.1.35-rust-1.59.0-slim-buster@sha256:b9809fc619b03e8fed8f235ea71408d40df86d7ab3d9586ef31c1a749bceac4f AS chef
# Cache-bust when this file changes
COPY rust-toolchain /
# See also: /rust-toolchain
RUN rustup toolchain install 1.59.0
RUN apt-get update -qq \
    && apt-get install --no-install-recommends -y \
      protobuf-compiler \
    && apt-get clean
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
