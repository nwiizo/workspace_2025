# cargo-chef ブログプロジェクト

このプロジェクトは、[前回の記事](https://syu-m-5151.hatenablog.com/entry/2025/10/17/070250)の続編として、cargo-chefの仕組みと動作原理を詳細に解説するブログ記事を管理しています。

## プロジェクト構成

```
cargo-chef-blog/
├── blog.md                    # メインのブログ記事（完全版）
├── contents/                  # セクションごとに分割された記事
│   ├── 00-introduction/       # はじめに
│   ├── 01-problem/            # RustのDockerビルドにおける根本的な問題
│   ├── 02-why-rust/           # Rustのビルドが特に問題になる理由
│   ├── 03-architecture/       # 技術スタックとアーキテクチャ
│   ├── 04-recipe-concept/     # レシピコンセプトと動作原理
│   ├── 05-internal-mechanism/ # 動作原理と内部メカニズム
│   ├── 06-docker-integration/ # Docker統合とマルチステージビルド
│   ├── 07-build-integration/  # ビルドプロセスの統合とサポート機能
│   ├── 08-use-cases/          # 主要機能と実用的なユースケース
│   ├── 09-performance/        # パフォーマンス特性と測定結果
│   ├── 10-verification/       # 実測検証
│   ├── 11-limitations/        # 重要な制限と考慮事項
│   ├── 12-design-patterns/    # 設計パターンとアーキテクチャの決定
│   ├── 13-alternatives/       # 代替アプローチとの比較
│   ├── 14-ecosystem/          # エコシステムとリソース
│   └── 15-conclusion/         # まとめとベストプラクティス
├── sample-api/                # 検証用サンプルアプリケーション
├── split_blog.py              # ブログをセクションに分割するPythonスクリプト
├── update-contents.sh         # セクション分割の自動化スクリプト
└── README.md                  # このファイル
```

## 使い方

### 1. ブログ記事の編集

メインのブログ記事を編集：

```bash
# お好みのエディタで編集
vim blog.md
```

### 2. セクションへの自動分割

blog.mdを更新したら、以下のコマンドでcontents/配下のセクションを自動更新：

```bash
./update-contents.sh
```

このスクリプトは自動的に：
- contents/ディレクトリをクリーンアップ
- blog.mdをセクションごとに分割
- 各セクションにblog.mdとREADME.mdを生成

### 3. セクションの確認

特定のセクションを確認：

```bash
cat contents/01-problem/blog.md
```

すべてのセクションを一覧表示：

```bash
ls -la contents/
```

### 4. 完全版の再構成

分割されたセクションから完全版を再構成：

```bash
cat contents/*/blog.md > blog-complete.md
```

## スクリプト詳細

### split_blog.py

Pythonスクリプトでblog.mdをセクションごとに分割します。

**特徴：**
- セクション（## で始まる行）を自動検出
- 各セクションを個別のディレクトリに配置
- README.mdを自動生成
- 日本語タイトルを適切に処理

### update-contents.sh

split_blog.pyのラッパースクリプトで、より使いやすいインターフェースを提供します。

**特徴：**
- 進捗状況の表示
- エラーチェック
- 各セクションの行数表示

## ワークフロー

1. **編集**: blog.mdを編集
2. **分割**: `./update-contents.sh`を実行
3. **確認**: contents/配下のセクションを確認
4. **公開**: 必要に応じてブログに投稿

## 技術スタック

- **言語**: Markdown
- **自動化**: Python 3, Bash
- **検証用サンプル**: Rust (actix-web)

## 前回の記事との関連

この記事は、[Rust Docker イメージサイズを 98%削減する方法](https://syu-m-5151.hatenablog.com/entry/2025/10/17/070250)の続編です。前回の記事でcargo-chefを使用したDockerビルドの最適化を紹介しましたが、この記事ではcargo-chefの内部動作と詳細な仕組みを解説しています。

## ライセンス

このドキュメントは情報共有を目的としています。
