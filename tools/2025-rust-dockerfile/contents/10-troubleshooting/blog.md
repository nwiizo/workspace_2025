## よくある問題と解決策

### OpenSSLリンクエラー

**エラー：** "Could not find directory of OpenSSL installation"

**解決策1：vendored OpenSSL（最も簡単）**

```toml
[dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

**解決策2：Alpine適切パッケージ**

```dockerfile
FROM rust:1.85-alpine
RUN apk add --no-cache openssl-dev openssl-libs-static musl-dev
```

**解決策3：Debianベース使用**

```dockerfile
FROM rust:1.85-slim-bookworm
RUN apt-get update && apt-get install -y pkg-config libssl-dev
```

**解決策4：rustls（Rust-native TLS）**

```toml
[dependencies]
reqwest = { version = "0.11", features = ["rustls-tls"], default-features = false }
```

### muslリンクエラー

**Alpine向け：**

```dockerfile
FROM rust:1.85-alpine
RUN apk add musl-dev openssl-dev openssl-libs-static
RUN rustup target add x86_64-unknown-linux-musl
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN cargo build --release --target x86_64-unknown-linux-musl
```

**必要な環境変数：**
- `RUSTFLAGS='-C target-feature=+crt-static'`
- `PKG_CONFIG_ALLOW_CROSS=1`
- `OPENSSL_STATIC=1`（システムOpenSSL使用時）

### DNS解決エラー（scratchイメージ）

**解決策1：distroless/static使用**

```dockerfile
FROM gcr.io/distroless/static-debian12
```

**解決策2：Pure Rust DNSリゾルバー**

```toml
[dependencies]
reqwest = { version = "0.11", features = ["trust-dns"] }
```

**解決策3：必要ファイルコピー**

```dockerfile
FROM alpine:latest AS ca-certificates
RUN apk add -U --no-cache ca-certificates

FROM scratch
COPY --from=ca-certificates /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
```

### .dockerignoreの重要性

**.dockerignoreがないと、`target/`ディレクトリ（数GB）がビルドコンテキストに含まれ、ビルドが遅くなります**。

```
# .dockerignore
target/
.git/
.env
*.log
```

**効果**: ビルドコンテキストのサイズを数GBから数MBに削減 → ビルド開始が高速化。

### イメージサイズ肥大化

**一般的原因と解決策：**

1. **最終イメージにビルドツール含む** → マルチステージビルドにより93%削減
2. **本番環境でfull rustイメージを使用** → slimランタイムベースにより95%削減
3. **バイナリにデバッグシンボルが含まれる** → `strip target/release/myapp`により30-40%削減
4. **開発依存関係** → プロファイル設定

```toml
[profile.release]
strip = true
lto = true
codegen-units = 1
```

