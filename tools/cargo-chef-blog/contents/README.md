# cargo-chef ブログコンテンツ

このディレクトリには、cargo-chefブログ記事が各セクションごとに分割されて格納されています。

## 構造

```
contents/
├── 00-introduction/          # はじめに
├── 01-problem/               # RustのDockerビルドにおける根本的な問題
├── 02-why-rust/              # Rustのビルドが特に問題になる理由
├── 03-architecture/          # 技術スタックとアーキテクチャ
├── 04-recipe-concept/        # レシピコンセプトと動作原理
├── 05-internal-mechanism/    # 動作原理と内部メカニズム
├── 06-docker-integration/    # Docker統合とマルチステージビルド
├── 07-build-integration/     # ビルドプロセスの統合とサポート機能
├── 08-use-cases/             # 主要機能と実用的なユースケース
├── 09-performance/           # パフォーマンス特性と測定結果
├── 10-verification/          # 実測検証
├── 11-limitations/           # 重要な制限と考慮事項
├── 12-design-patterns/       # 設計パターンとアーキテクチャの決定
├── 13-alternatives/          # 代替アプローチとの比較
├── 14-ecosystem/             # エコシステムとリソース
└── 15-conclusion/            # まとめとベストプラクティス
```

各ディレクトリには以下のファイルが含まれています：

- `blog.md`: セクションの本文
- `README.md`: セクションの説明

## 全体を結合する

全てのセクションを結合して完全なブログ記事を生成するには：

```bash
cat 00-introduction/blog.md \
    01-problem/blog.md \
    02-why-rust/blog.md \
    03-architecture/blog.md \
    04-recipe-concept/blog.md \
    05-internal-mechanism/blog.md \
    06-docker-integration/blog.md \
    07-build-integration/blog.md \
    08-use-cases/blog.md \
    09-performance/blog.md \
    10-verification/blog.md \
    11-limitations/blog.md \
    12-design-patterns/blog.md \
    13-alternatives/blog.md \
    14-ecosystem/blog.md \
    15-conclusion/blog.md \
    > ../blog-complete.md
```
