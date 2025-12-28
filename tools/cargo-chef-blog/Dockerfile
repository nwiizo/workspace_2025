# syntax=docker/dockerfile:1
# 2025年のRust最適化Dockerfile

ARG RUST_VERSION=1.85
ARG APP_NAME=myapp

# cargo-chef: 依存関係とソースコードを分離してキャッシュ最適化
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# BuildKitキャッシュマウント: レイヤーを超える永続的キャッシュ
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin ${APP_NAME} && \
    cp ./target/release/${APP_NAME} /bin/server

# distroless: 50MB、セキュア、非root実行
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /bin/server /app/
WORKDIR /app
EXPOSE 8000
ENTRYPOINT ["/app/server"]
