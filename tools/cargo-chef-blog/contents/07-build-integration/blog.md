# ビルドプロセスの統合とサポート機能

cargo-chef は標準的な Cargo ワークフローとシームレスに統合し、ビルドカスタマイズの全範囲をサポートします:

**ビルドコマンド:**

- `build`(デフォルト)
- `check`(--check フラグ経由)
- `clippy`
- `zigbuild`

**サポートされるオプション:**

- プロファイル選択: `--release`、`--debug`、カスタム `--profile`
- 機能: `--features`、`--no-default-features`、`--all-features`
- ターゲット: `--target`、`--target-dir`(ファーストクラスのクロスコンパイルサポート)
- ターゲットタイプ: `--benches`、`--tests`、`--examples`、`--all-targets`、`--bins`、`--bin`
- ワークスペース: `--workspace`、`--package`、`--manifest-path`
- Cargo フラグ: `--offline`、`--frozen`、`--locked`、`--verbose`、`--timings`
- ツールチェーンオーバーライド: `cargo +nightly chef cook`

**ワークスペースサポート**は自動です。cargo-chef はワークスペース内のすべてのクレートを検出し、正しく処理します。ファイルやクレートが移動しても、cargo-chef は自動的に適応します—Dockerfile の変更は不要です。これは、プロジェクト構造をハードコードする手動アプローチに対する大きな利点です。

**ビルド済みDockerイメージ**は Docker Hub の `lukemathwalker/cargo-chef` で利用可能で、柔軟なタグ付けができます。

- `latest-rust-1.75.0`(特定の Rust バージョンの最新 cargo-chef)
- `0.1.72-rust-latest`(最新の Rust の特定 cargo-chef)
- Alpine バリアント: `latest-rust-1.70.0-alpine3.18`

バージョンの一貫性: すべてのステージで**同一のRustバージョン**を使用すべきです。バージョンの不一致は、異なるコンパイラバージョンが異なるアーティファクトを生成するため、キャッシングを無効化します。
