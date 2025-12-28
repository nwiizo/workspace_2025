## ユースケース別Dockerfile

### Webアプリケーション（Axum / Actix-web）

上記の「標準Dockerfile」パターンをそのまま使用できます。

### CLIツール（完全静的リンク）

```dockerfile
# syntax=docker/dockerfile:1

FROM rust:1.85-alpine AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

# 依存関係のキャッシュ
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/cli-tool /app/
ENTRYPOINT ["/app/cli-tool"]
```

シェルエイリアスは以下のように設定できます。
```bash
alias my-cli='docker run --rm -v $(pwd):/data my-cli-image'
```

### ワークスペース（モノレポ）対応

```dockerfile
# syntax=docker/dockerfile:1

ARG SERVICE_NAME=api-gateway
ARG RUST_VERSION=1.85

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
ARG SERVICE_NAME
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo chef cook --release --bin ${SERVICE_NAME} --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin ${SERVICE_NAME} && \
    cp ./target/release/${SERVICE_NAME} /bin/service

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /bin/service /app/
ENTRYPOINT ["/app/service"]
```

異なるサービスを同じDockerfileから生成できます。
```bash
docker build --build-arg SERVICE_NAME=api-gateway -t gateway .
docker build --build-arg SERVICE_NAME=user-service -t users .
```

