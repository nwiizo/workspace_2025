# 主要機能と実用的なユースケース

**主なユースケース:**

**1. CI/CDパイプラインの最適化** - 標準的なユースケースです。すべてのコード変更が CI で Docker ビルドをトリガーします。cargo-chef なしでは、各ビルドが 500 以上のすべての依存関係を再コンパイルします(10〜20 分)。cargo-chef があれば、変更されていない依存関係はキャッシュされ、ビルドは 2〜3 分に短縮されます。これは次のような点に直接影響します。

- デプロイ速度(機能をより速くリリース)
- インシデント対応(本番環境をより速くパッチ)
- 開発者体験(PR へのより速いフィードバック)
- インフラコスト(消費される CPU 分の削減)

**2. マルチステージビルド** - ビルド環境とランタイム環境を分離。ビルダーステージは完全な Rust ツールチェーン(800MB 以上)を含み、ランタイムステージは最小イメージ(25〜50MB)を使用します。cargo-chef は、高コストなビルダーステージをキャッシュ状態に保つことで、このパターンを実用的にします。

**3. ワークスペース/モノレポプロジェクト** - 依存関係を共有する複数のバイナリとライブラリを自動的に処理します。手動アプローチはワークスペースで破綻します; cargo-chef は透過的に処理します。

**4. クロスコンパイル** - `--target` フラグ経由でファーストクラスサポート。例: Alpine Linux デプロイのために `x86_64-unknown-linux-musl` バイナリを CI でビルド。ターゲット指定は依存関係キャッシング中に尊重されます。

**高度な最適化戦略:**

**sccacheとの組み合わせ:**
```dockerfile
FROM rust:1.75 AS base
RUN cargo install --locked cargo-chef sccache
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# ... plannerステージ ...

FROM base AS builder
RUN --mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json
```

この組み合わせは**2層のキャッシング**を提供します。

- cargo-chef: 粗粒度(依存関係レイヤー全体)
- sccache: 細粒度(個々のコンパイルアーティファクト)

1 つの依存関係が変更された場合、cargo-chef はすべてを再ビルドしますが、sccache は個々のクレートコンパイルをキャッシュします。変更された依存関係のみが実際に再コンパイルされます。

**BuildKitキャッシュマウント:**
```dockerfile
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo chef cook --release --recipe-path recipe.json
```

これは cargo レジストリ自体をキャッシュし、再ダウンロードを回避します。sccache および cargo-chef と組み合わせることで、Rust Docker ビルドの現在のベストプラクティスとなります。
