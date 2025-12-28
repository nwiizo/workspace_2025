# 技術スタックとアーキテクチャ

cargo-chef は**Rustで書かれており**(edition 2018)、バイナリ(`cargo-chef`)とライブラリ(`chef`)の両方として配布されています。現在のバージョンは 0.1.72 で、Apache 2.0 と MIT のデュアルライセンスです。

**コア依存関係:**

- **clap v4** - cargo/derive 機能を使用した CLI 引数パース
- **serde v1.0**および**serde_json v1.0** - レシピの JSON 形式へのシリアライズ
- **toml v0.8** - `preserve_order` 機能で Cargo.toml マニフェストをパース
- **cargo-manifest v0.18**および**cargo_metadata v0.15** - Cargo ワークスペース構造へのプログラマティックアクセス
- **fs-err v2.5**、**globwalk v0.8**、**pathdiff v0.2** - ファイルシステム操作
- **anyhow v1.0** - エラーハンドリング
- **log v0.4**および**env_logger v0.10** - ロギング

**プロジェクト構造:**

- `src/main.rs` - コマンドパースを含む CLI エントリポイント
- `src/lib.rs` - ライブラリエントリポイント
- `src/recipe.rs` - レシピ生成、依存関係ビルド、クッキングロジック
- `src/skeleton.rs` - プロジェクトスケルトンの作成とダミーファイル生成

アーキテクチャは 2 つの抽象化を中心としています: **Recipe**と**Skeleton**。Recipe はシリアライズ可能なコンテナで、Skeleton は実際のマニフェストデータとロックファイルを含みます。これらの構造により、コアワークフローが可能になります: 分析 → シリアライズ → 再構築 → ビルド。
