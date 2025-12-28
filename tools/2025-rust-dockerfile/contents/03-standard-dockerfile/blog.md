## 標準Dockerfile：2025年の推奨パターン

2025年のRust標準Dockerfileです。これがすべてのベストプラクティスを含んだテンプレートです。

```dockerfile
# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.85
ARG APP_NAME=myapp

# cargo-chefを使った依存関係キャッシング
FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
# 依存関係のビルド（キャッシュ可能）
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# アプリケーションのビルド
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin ${APP_NAME} && \
    cp ./target/release/${APP_NAME} /bin/server

# テストステージ（オプション）
FROM chef AS test
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo test

# 本番ステージ：distroless
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
COPY --from=builder /bin/server /app/
WORKDIR /app
EXPOSE 8000
ENTRYPOINT ["/app/server"]
```

**このDockerfileの主な特徴**

- **cargo-chef**による依存関係の分離とキャッシング
- **BuildKitキャッシュマウント**でレイヤーを跨いだキャッシュ
- **distroless**による最小サイズと高セキュリティ
- **非rootユーザー**（`:nonroot`タグ）での実行
- オプショナルな**テストステージ**

