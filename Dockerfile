# https://github.com/LukeMathWalker/cargo-chef/tree/v0.1.31#how-to-use
# https://github.com/LukeMathWalker/cargo-chef/tree/v0.1.31#pre-built-images
FROM lukemathwalker/cargo-chef:latest-rust-1.55.0@sha256:e70c3dc65a557a5a862947de322f9ad1198abea1a7f208f6d16ce29ff58e5859 AS chef
# Cache-bust when this file changes
COPY rust-toolchain /
# See also: /rust-toolchain
RUN rustup toolchain update stable
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN rm -rf .cargo && \
    SQLX_OFFLINE=true \
    cargo build --release

FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/axum-rest-example /usr/local/bin
COPY config /app/config
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/axum-rest-example"]
