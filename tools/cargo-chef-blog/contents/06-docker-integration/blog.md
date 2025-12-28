# Docker統合とマルチステージビルド

cargo-chef は、Docker のマルチステージビルド機能用に特別に設計されています。標準的なパターンは 3 つのステージを使用します:

**標準的な3ステージパターン:**
```dockerfile
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# ステージ1: Planner - レシピを生成
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ステージ2: Builder - 依存関係をキャッシュ
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
# ↑ このレイヤーは依存関係が変更されるまでキャッシュされる

# 次にソースをコピーしてアプリケーションをビルド
COPY . .
RUN cargo build --release --bin app

# ステージ3: Runtime - 最小限の本番イメージ
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/app /usr/local/bin
ENTRYPOINT ["/usr/local/bin/app"]
```

**キャッシングの仕組み:**

各 Docker ステージは独立したキャッシングを維持します。ステージは `COPY --from` 文を通じてのみやり取りします。この分離が cargo-chef の効果の鍵です。

1. planner ステージの `COPY . .` は planner キャッシュを無効化(ただしこれは高速)
2. Planner はフルソースツリーから recipe.json を生成
3. Builder ステージは `COPY --from=planner` 経由で recipe.json のみを受け取る
4. **recipe.jsonのチェックサムが変更されていない限り、builderの依存関係レイヤーはキャッシュされたまま**
5. Cargo.toml または Cargo.lock が変更された場合にのみ recipe.json が変更される
6. ソースコードの変更は recipe.json に影響しないため、依存関係レイヤーはキャッシュされたまま

**キャッシュ無効化ロジック:**
```
ソースコード変更 → plannerステージ無効化
                → recipe.json変更なし
                → builderの依存関係レイヤーキャッシュ済み ✓
                → アプリケーションビルドのみ実行

依存関係変更    → plannerステージ無効化
                → recipe.json変更
                → builderの依存関係レイヤー無効化 ✗
                → フルリビルド必要
```

これはインセンティブを完璧に整合させます: 高コストな操作(依存関係コンパイル)は、そうあるべき時(依存関係が変更されていない時)にキャッシュされ、高速な操作(ソースコンパイル)は期待通り毎回の変更で実行されます。
