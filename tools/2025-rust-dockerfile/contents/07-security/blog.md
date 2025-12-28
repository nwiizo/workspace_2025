## セキュリティベストプラクティス

### 1. 非rootユーザーで実行

**distroless `:nonroot`タグが最も簡単：**

```dockerfile
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/myapp /usr/local/bin/
CMD ["/usr/local/bin/myapp"]
```

自動的にUID 65534（nobody）として実行されます。

**カスタムユーザー作成：**

```dockerfile
FROM debian:bookworm-slim

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

USER appuser
COPY --from=builder /app/target/release/myapp /app/
CMD ["/app/myapp"]
```

### 2. 脆弱性スキャン

**Trivy（推奨）：**

```bash
# イメージスキャン
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    aquasec/trivy image myapp:latest

# CI/CD統合
- name: Run Trivy scan
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: 'myapp:${{ github.sha }}'
    severity: 'CRITICAL,HIGH'
    exit-code: '1'
```

[https://github.com/aquasecurity/trivy:embed:cite]

**distrolessのセキュリティ優位性：**
- Alpine（musl）からChiseled Ubuntu（glibc）への移行で30+ CVEが0 CVEに
- distrolessイメージはAlpineより**50-80%少ないCVE**
- パッケージマネージャー不在により攻撃ベクトル削減
- SLSA 2準拠、cosign署名認証

### 3. シークレット管理

**絶対に避けるべき：環境変数へのシークレット設定**

イメージに焼き込まれてしまいます。

**正しい方法：**

```dockerfile
# ビルド時シークレット
RUN --mount=type=secret,id=api_token,env=API_TOKEN \
    cargo build --release
```

```bash
# 実行時
docker build --secret id=api_token,env=API_TOKEN .
```

### 4. イメージバージョンのピン留め

```dockerfile
# ❌ 避けるべき
FROM rust:latest

# ✅ 推奨
FROM rust:1.85-slim-bookworm
```

