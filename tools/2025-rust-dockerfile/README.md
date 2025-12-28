# Rust Docker Demo 2025

2025年のベストプラクティスに基づいたRust Dockerfileの実装例です。

## プロジェクト構成

```
.
├── Cargo.toml          # プロジェクト定義と依存関係
├── Dockerfile          # 2025年推奨のマルチステージDockerfile
├── .dockerignore       # ビルドコンテキストから除外するファイル
├── src/
│   └── main.rs         # Axum Webアプリケーション
└── README.md           # このファイル
```

## アプリケーション機能

シンプルなREST APIサーバー（Axum使用）：

- `GET /` - ウェルカムメッセージ
- `GET /health` - ヘルスチェック
- `POST /users` - ユーザー作成（デモ）

## ローカル開発

### 依存関係の確認

```bash
cargo check
```

### ビルド

```bash
cargo build --release
```

### テスト実行

```bash
cargo test
```

### ローカル実行

```bash
cargo run
# または
./target/release/rust-docker-demo
```

サーバーは http://localhost:8000 で起動します。

### 動作確認

```bash
# ルートエンドポイント
curl http://localhost:8000/

# ヘルスチェック
curl http://localhost:8000/health

# ユーザー作成
curl -X POST http://localhost:8000/users \
  -H "Content-Type: application/json" \
  -d '{"name":"John Doe","email":"john@example.com"}'
```

## Dockerビルド

### 2025年ベストプラクティス Dockerfile の特徴

1. **cargo-chef による依存関係キャッシング**
   - 依存関係とソースコードのビルドを分離
   - 依存関係が変更されない限りキャッシュを活用

2. **BuildKit キャッシュマウント**
   - `--mount=type=cache` でレイヤーを超えたキャッシュ
   - `/usr/local/cargo/registry` - crates.io ダウンロード
   - `/usr/local/cargo/git` - Git 依存関係
   - `/app/target` - ビルド成果物

3. **distroless 最終イメージ**
   - `gcr.io/distroless/cc-debian12:nonroot` 使用
   - イメージサイズ約29MB
   - 非rootユーザーで実行（セキュリティ向上）
   - シェルやパッケージマネージャーなし（攻撃面削減）

4. **マルチステージビルド**
   - planner: recipe.json 生成
   - builder: アプリケーションビルド
   - test: テスト実行（オプション）
   - runtime: 最終成果物

### Docker イメージのビルド

```bash
# 標準ビルド
docker build -t rust-docker-demo:latest .

# テストステージも実行
docker build --target test -t rust-docker-demo:test .
```

### Docker コンテナの実行

```bash
# バックグラウンド実行
docker run -d -p 8000:8000 --name rust-demo rust-docker-demo:latest

# フォアグラウンド実行
docker run --rm -p 8000:8000 rust-docker-demo:latest

# 環境変数でポート変更
docker run --rm -p 3000:3000 -e PORT=3000 rust-docker-demo:latest
```

### イメージサイズの確認

```bash
docker images rust-docker-demo:latest
```

期待されるイメージサイズ: 約30-40MB

### キャッシュの効果確認

```bash
# 初回ビルド（すべてビルド）
time docker build -t rust-docker-demo:v1 .

# ソースコード変更後の再ビルド（依存関係キャッシュ利用）
# src/main.rs を少し変更してから
time docker build -t rust-docker-demo:v2 .
```

初回: 10-15分 → 再ビルド: 1-2分 の高速化が期待できます。

## マルチアーキテクチャビルド

```bash
# ビルダーの作成
docker buildx create --name container-builder \
    --driver docker-container --bootstrap --use

# AMD64 と ARM64 の両方でビルド
docker buildx build \
    --platform linux/amd64,linux/arm64 \
    -t rust-docker-demo:latest \
    --push .
```

## CI/CD統合例（GitHub Actions）

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

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

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

## パフォーマンスベンチマーク

### ビルド時間（参考値）

| シナリオ | 時間 |
|---------|------|
| 初回ビルド（依存関係含む） | 10-15分 |
| ソースコード変更後 | 1-2分 |
| 依存関係更新後 | 5-8分 |

### イメージサイズ比較

| イメージ | サイズ |
|---------|--------|
| rust:1.85（開発用） | 1.8GB |
| debian:bookworm-slim（ランタイム） | 80-120MB |
| distroless/cc-debian12 | **29MB** |
| scratch + musl | 8-15MB |

## トラブルシューティング

### OpenSSL リンクエラー

依存関係に `openssl` がある場合：

```toml
# Cargo.toml
[dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

### DNS解決エラー

scratchイメージではなくdistrolessを使用することで解決。

## 参考資料

- [blog.md](./blog.md) - 詳細な解説とベンチマーク
- [cargo-chef](https://github.com/LukeMathWalker/cargo-chef)
- [distroless](https://github.com/GoogleContainerTools/distroless)
- [BuildKit](https://docs.docker.com/build/buildkit/)

## ライセンス

MIT
