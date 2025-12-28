## マルチアーキテクチャビルド

linux/amd64とlinux/arm64の両対応が2025年の標準要件です。

### cargo-zigbuild：セットアップゼロのクロスコンパイル

**cargo-zigbuild（v0.20.1）**はZigツールチェインを使い、セットアップ不要でクロスコンパイルできます。

[https://github.com/rust-cross/cargo-zigbuild:embed:cite]

```dockerfile
# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.85

FROM --platform=$BUILDPLATFORM rust:${RUST_VERSION}-alpine AS builder
WORKDIR /app

# Zigとcargo-zigbuildのインストール
RUN apk add --no-cache musl-dev openssl-dev zig
RUN cargo install --locked cargo-zigbuild

# ターゲットの設定
ARG TARGETPLATFORM
RUN case ${TARGETPLATFORM} in \
    "linux/amd64") echo x86_64-unknown-linux-musl > /rust_target ;; \
    "linux/arm64") echo aarch64-unknown-linux-musl > /rust_target ;; \
    esac && \
    rustup target add $(cat /rust_target)

# 依存関係とビルド
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo zigbuild --release --target $(cat /rust_target) && \
    rm -rf src

COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo zigbuild --release --target $(cat /rust_target)

FROM alpine:latest
ARG TARGETPLATFORM
COPY --from=builder /app/target/*/release/app /app
CMD ["/app"]
```

**重要：`--platform=$BUILDPLATFORM`により、ビルド自体はネイティブアーキテクチャで実行**されるため、QEMUエミュレーションより圧倒的に高速です（QEMUエミュレーションは16-25倍遅い）。

**実測データ：**
- ネイティブビルド：2-3分
- QEMUエミュレーション：50分（**16-25倍遅い**）
- cargo-zigbuildクロスコンパイル：13分

### Docker buildxでのマルチプラットフォームビルド

```bash
# ビルダーの作成
docker buildx create --name container-builder \
    --driver docker-container --bootstrap --use

# マルチプラットフォームビルド
docker buildx build \
    --platform linux/amd64,linux/arm64 \
    -t myimage:latest \
    --push .
```

[https://docs.docker.com/build/buildx/:embed:cite]

### GitHub Actionsでのマルチアーキテクチャビルド

```yaml
name: Build and Push

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: true
          tags: ghcr.io/${{ github.repository }}:latest
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

[https://docs.github.com/en/actions:embed:cite]
[https://github.com/docker/build-push-action:embed:cite]

