# RustのDockerfile、2025年はこれでいこう - Contents

このディレクトリには、ブログ記事のセクション別コンテンツが含まれています。

## ディレクトリ構造

```
contents/
├── 00-introduction/            # はじめに
│   ├── blog.md
│   └── README.md
├── 01-updates-2025/            # 2025年の重要なアップデート
│   ├── blog.md
│   └── README.md
├── 02-mental-model/            # 基本的な考え方
│   ├── blog.md
│   └── README.md
├── 03-standard-dockerfile/     # 標準Dockerfile
│   ├── blog.md
│   └── README.md
├── 04-build-optimization/      # ビルド最適化の3つの柱
│   ├── blog.md
│   └── README.md
├── 05-image-size/              # イメージサイズの最適化
│   ├── blog.md
│   └── README.md
├── 06-multi-arch/              # マルチアーキテクチャビルド
│   ├── blog.md
│   └── README.md
├── 07-security/                # セキュリティベストプラクティス
│   ├── blog.md
│   └── README.md
├── 08-usecases/                # ユースケース別Dockerfile
│   ├── blog.md
│   └── README.md
├── 09-verification/            # 実践的な検証結果
│   ├── blog.md
│   └── README.md
├── 10-troubleshooting/         # よくある問題と解決策
│   ├── blog.md
│   └── README.md
├── 11-new-tools/               # 2025年の新ツール活用
│   ├── blog.md
│   └── README.md
├── 12-performance/             # パフォーマンス比較とROI
│   ├── blog.md
│   └── README.md
├── 13-summary/                 # まとめ
│   ├── blog.md
│   └── README.md
└── 14-closing/                 # おわりに
    ├── blog.md
    └── README.md
```

## セクション一覧

| # | タイトル | ディレクトリ |
|---|---------|------------|
| 00 | はじめに | `00-introduction/` |
| 01 | 2025年の重要なアップデート | `01-updates-2025/` |
| 02 | 基本的な考え方：2025年のメンタルモデル | `02-mental-model/` |
| 03 | 標準Dockerfile：2025年の推奨パターン | `03-standard-dockerfile/` |
| 04 | ビルド最適化の3つの柱 | `04-build-optimization/` |
| 05 | イメージサイズの最適化 | `05-image-size/` |
| 06 | マルチアーキテクチャビルド | `06-multi-arch/` |
| 07 | セキュリティベストプラクティス | `07-security/` |
| 08 | ユースケース別Dockerfile | `08-usecases/` |
| 09 | 実践的な検証結果 | `09-verification/` |
| 10 | よくある問題と解決策 | `10-troubleshooting/` |
| 11 | 2025年の新ツール活用 | `11-new-tools/` |
| 12 | パフォーマンス比較とROI | `12-performance/` |
| 13 | まとめ：2025年のゴールデンルール | `13-summary/` |
| 14 | おわりに | `14-closing/` |

## 使い方

### 全体を読む
完全版は `../blog_full.md` を参照してください。

### セクション別に読む
各ディレクトリの `blog.md` を参照してください。

### 自動更新
blog_full.md を更新した後、以下のコマンドで再生成できます:

```bash
./split_blog.sh
```

## 自動化

このディレクトリ構造は `split_blog.sh` スクリプトによって自動生成されています。
手動で編集した内容はスクリプト実行時に上書きされるため、編集は `blog_full.md` に対して行ってください。
