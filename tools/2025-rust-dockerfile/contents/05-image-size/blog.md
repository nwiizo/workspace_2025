## イメージサイズの最適化：ベースイメージ選択戦略

### ビルドステージ：rust:slim推奨

**rust:slim（Debian系）**が2025年の標準推奨です。

```dockerfile
FROM rust:1.85-slim-bookworm AS builder
```

**理由：**
- Debian stable（bookworm）ベースでglibcを使用
- 広範な互換性とマルチスレッドワークロードでの優れたパフォーマンス
- 完全版rust:latestの624MBに対し、コンパイルに必要な最小限のパッケージのみ

**rust:alpineは避けるべき：**
- muslの互換性問題
- **マルチスレッドアプリケーションで最大30倍のパフォーマンス劣化**

[https://hub.docker.com/_/rust:embed:cite]

### 最終ステージ：distroless推奨

**gcr.io/distroless/cc-debian12が2025年の標準推奨**です。

```dockerfile
FROM gcr.io/distroless/cc-debian12:nonroot
```

**distrolessの特徴：**
- サイズ：21-29MB
- glibc、SSL証明書、タイムゾーンデータ、/etc/passwdを含む
- パッケージマネージャー、シェル不要なバイナリを完全排除
- SLSA 2準拠、cosign署名検証が可能
- CVEスキャンで従来イメージより**50-80%少ない脆弱性**
- `:nonroot`タグでUID 65534（nobody）として非rootで実行

[https://github.com/GoogleContainerTools/distroless:embed:cite]

### イメージサイズ比較（実測値）

| イメージ構成 | サイズ | 用途 | 特徴 |
|-------------|--------|------|------|
| **scratch + musl（実測）** | **1.71MB** | CLIツール最小化 | 完全静的リンク |
| distroless/static | 2-3MB | 静的リンクバイナリ | 最小限のファイル |
| **distroless/cc-debian12（実測）** | **50.3MB** | **Webアプリ推奨** | glibc |
| debian-slim | 80-120MB | フル互換性 | デバッグツールあり |
| rust:latest（未最適化） | 2.63GB | 開発専用 | ビルドツール込み |

**実測削減率：**
- rust:latest（2.63GB）→ distroless（50.3MB）：**98.1%削減**
- rust:latest（2.63GB）→ musl+scratch（1.71MB）：**99.9%削減**

### 静的リンク vs 動的リンク

**musl（x86_64-unknown-linux-musl）での静的リンク：**

```dockerfile
FROM rust:1.85-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app

# 依存関係のキャッシュ
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# アプリケーションのビルド
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/myapp /myapp
ENTRYPOINT ["/myapp"]
```

**利点：**
- 依存関係ゼロで完全にポータブル
- scratchコンテナで実行可能
- イメージサイズ5-10MB

**欠点：**
- **シングルスレッドで0.9-1.0倍、マルチスレッドで0.03-0.5倍のパフォーマンス**
- 一部依存関係でsegfaultのリスク

**本番環境の推奨：**
- 複雑なアプリケーション（Webサーバー、DB接続）：**glibc + distroless/cc-debian12**
- シンプルなCLIツール：musl + scratchを検討
- パフォーマンスが重要：**必ずglibcを使用**

