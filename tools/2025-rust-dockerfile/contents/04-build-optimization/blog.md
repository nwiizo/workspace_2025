## ビルド最適化の3つの柱

### 1. cargo-chef：依存関係キャッシングの決定版

cargo-chefは、Rustの依存関係管理をDockerレイヤーキャッシュに適合させる画期的なツールです。**依存関係のコンパイルとソースコードのコンパイルを完全に分離**します。

**動作メカニズム（2段階）：**

1. `cargo chef prepare`：Cargo.tomlとCargo.lockを解析してrecipe.jsonを作成
2. `cargo chef cook`：最小限のプロジェクト構造を再構築して依存関係のみをビルド

**重要：同じRustバージョンと作業ディレクトリを全ステージで使用すること**。異なるバージョンを使うとキャッシュが無効化されます。

**実測データ：**
- cargo-chefのみで55%の改善
- cargo-chef + sccacheで**79%の改善**（34秒→7秒）
- 商用プロジェクト（14,000行、500依存関係）で10分→2分

[https://github.com/LukeMathWalker/cargo-chef:embed:cite]
[https://docs.rs/crate/cargo-chef/latest:embed:cite]

### 2. BuildKitキャッシュマウント：レイヤーを超えるキャッシュ

BuildKitのキャッシュマウント（Docker 18.09以降）を使うと、レイヤー無効化を超えて永続化するキャッシュボリュームが利用できます。

**3つの重要なキャッシュポイント：**

```dockerfile
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release
```

- `/usr/local/cargo/registry`：crates.ioからのダウンロード
- `/usr/local/cargo/git`：Git依存関係
- `/app/target`：ビルド成果物

**`sharing=locked`パラメータは排他アクセスを保証**し、パッケージマネージャーの破損を防ぎます。

**CI環境でのキャッシュ共有：**

```yaml
# GitHub Actions
- uses: docker/build-push-action@v6
  with:
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

**パフォーマンスベンチマーク：**
- ベースライン：90.60秒
- BuildKitキャッシュマウント：**15.63秒（5.8倍高速）**
- cargo-chef：18.81秒（4.8倍高速）
- 三位一体（chef + BuildKit + sccache）：**7-12秒（7.5-13倍高速）**

[https://docs.docker.com/build/buildkit/:embed:cite]

### 3. sccache：細粒度コンパイルキャッシュ

sccache（v0.7.x）はMozilla製のccache風コンパイララッパーで、**個々のコンパイル成果物を細粒度でキャッシュ**します。

[https://github.com/mozilla/sccache:embed:cite]

```dockerfile
FROM rust:1.85 AS builder

# sccacheのインストールと設定
RUN cargo install sccache --version ^0.7
ENV RUSTC_WRAPPER=sccache \
    SCCACHE_DIR=/sccache \
    CARGO_INCREMENTAL=0

WORKDIR /app

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    --mount=type=bind,target=. \
    cargo build --release
```

**重要：`CARGO_INCREMENTAL=0`は必須**。インクリメンタルコンパイルとsccacheは競合します。

**キャッシュヒット率：**
- 初回ビルド：0%
- ソースコード変更のみ：85-95%
- 依存関係を更新した時：60-75%

**注意点：**
sccacheの効果は環境によって大きく異なります。一部の環境では効果が薄く、逆にオーバーヘッドとなる場合があります。**自環境でのベンチマークが必須**です。

