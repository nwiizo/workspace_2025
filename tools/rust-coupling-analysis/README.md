# Rust Coupling Analysis Tool

A command-line tool for analyzing coupling in Rust projects based on Vlad Khononov's "Balancing Coupling in Software Design" framework.

## 概要

このツールは、Rustプロジェクトの結合度を分析し、保守性の高い設計を支援します。結合度を次の3つの次元で測定します：

1. **Integration Strength（統合強度）** - コンポーネント間で共有される知識の量
2. **Distance（距離）** - 依存関係がどれだけ離れているか
3. **Volatility（変動性）** - コンポーネントの変更頻度

## 機能

- ✅ **AST解析** - Rustコードの構文木を解析して結合パターンを検出
- ✅ **Git履歴分析** - 変更頻度から変動性を測定
- ✅ **バランススコア計算** - Khononovのフレームワークに基づくスコアリング
- ✅ **問題検出** - 結合度に関する問題を自動検出
- ✅ **Markdownレポート生成** - 分析結果を見やすいレポートで出力

## インストール

```bash
cargo install --path .
```

## 使い方

### 基本的な使用方法

```bash
# プロジェクトを分析してレポートを標準出力
rust-coupling-analysis ./src

# レポートをファイルに出力
rust-coupling-analysis --output report.md ./src

# サマリーのみ表示
rust-coupling-analysis --summary ./src
```

### オプション

```
OPTIONS:
    -h, --help           ヘルプ情報を表示
    -o, --output <FILE>  レポートをファイルに出力
    -s, --summary        サマリーのみ表示
    -v, --version        バージョン情報を表示
```

## 実装の詳細

### モジュール構成

- **analyzer** - AST訪問による結合度分析
- **metrics** - メトリクスデータ構造
- **balance** - バランススコア計算と問題検出
- **volatility** - Git履歴からの変動性分析
- **report** - Markdownレポート生成

### 結合度の分類

#### Integration Strength（統合強度）

- **Intrusive Coupling（侵入的結合）** - 内部実装の詳細に依存（最強）
- **Functional Coupling（機能的結合）** - 共有された責任による依存
- **Model Coupling（モデル結合）** - ビジネスドメインモデルの共有
- **Contract Coupling（契約結合）** - インターフェース/トレイトによる抽象化（最弱）

#### Distance（距離）

- 同じ関数内
- 同じモジュール内
- 異なるモジュール
- 異なるクレート
- 異なるサービス

#### Volatility（変動性）

- **Low（低）** - 0-2回の変更（汎用サブドメイン）
- **Medium（中）** - 3-10回の変更（サポートサブドメイン）
- **High（高）** - 11回以上の変更（コアサブドメイン）

### バランスの方程式

```
BALANCE = (STRENGTH XOR DISTANCE) OR NOT VOLATILITY
```

理想的な状態：
- 強い結合なら距離を近く（局所性を保つ）
- 弱い結合なら距離を遠くても良い（疎結合）
- モジュラーである、または変動性が低い

## テスト

```bash
# すべてのテストを実行
cargo test

# ドキュメントテストを実行
cargo test --doc
```

## 開発

### 依存関係

- `syn` - Rust構文解析
- `walkdir` - ディレクトリ走査
- `thiserror` - エラーハンドリング

### ビルド

```bash
# デバッグビルド
cargo build

# リリースビルド
cargo build --release
```

## ブログ記事

本ツールの解説記事が2つのバージョンで用意されています：

### 📄 blog.md - 簡潔版（推奨）
読みやすさを重視した簡潔なバージョン。約420行。

**特徴:**
- TL;DRで要点を把握
- 実装の核心部分のみ掲載
- 実行例とテスト結果を含む
- Good/Bad パターン集
- 10分で読める

### 📚 blog_full.md - 詳細版
完全な実装コードと詳細な説明を含む詳細版。約1,170行。

**特徴:**
- すべてのコード実装を掲載
- Connascenceの詳細説明
- 既存ツールの網羅的解説
- 全テストコードを含む
- じっくり学習したい方向け

### セクション分割版

どちらのバージョンも、セクションごとに分割された `contents/` ディレクトリで読めます。

```bash
# blog.mdを自動的にセクションごとに分割
python3 split_blog.py
```

**簡潔版の構成（17セクション）:**
1. TL;DR
2. なぜ結合度を測るのか
3. Balancing Couplingフレームワーク
4. 実装：30分で作る結合度分析ツール
5. 実行例：自己分析
6. テスト結果
7. 実践パターン集
8. まとめ
... など

**詳細版の構成（13セクション）:**
- より深い理論的背景
- 完全な実装コード
- 既存ツールの詳細比較
... など

## 参考文献

- Vlad Khononov - "Balancing Coupling in Software Design"
- Meilir Page-Jones - Connascence概念の提唱者
- Domain-Driven Design (DDD)

## ライセンス

MIT License

## 貢献

プルリクエストを歓迎します！以下の手順に従ってください：

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## 連絡先

問題や提案がある場合は、GitHubのIssueを作成してください。
