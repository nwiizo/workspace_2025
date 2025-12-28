#!/usr/bin/env bash
set -euo pipefail

# RustのDockerfile、2025年はこれでいこう - Blog Section Splitter
# blog_full.md を各セクションごとに contents/ 配下に配置

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly BLOG_FILE="${SCRIPT_DIR}/blog_full.md"
readonly CONTENTS_DIR="${SCRIPT_DIR}/contents"

# Clean up existing contents directory
rm -rf "${CONTENTS_DIR}"
mkdir -p "${CONTENTS_DIR}"

echo "📚 Splitting blog_full.md into sections..."

# Section definitions (section_number:directory_name:title)
declare -a SECTIONS=(
    "00:introduction:はじめに"
    "01:updates-2025:2025年の重要なアップデート"
    "02:mental-model:基本的な考え方：2025年のメンタルモデル"
    "03:standard-dockerfile:標準Dockerfile：2025年の推奨パターン"
    "04:build-optimization:ビルド最適化の3つの柱"
    "05:image-size:イメージサイズの最適化：ベースイメージ選択戦略"
    "06:multi-arch:マルチアーキテクチャビルド"
    "07:security:セキュリティベストプラクティス"
    "08:usecases:ユースケース別Dockerfile"
    "09:verification:実践的な検証結果"
    "10:troubleshooting:よくある問題と解決策"
    "11:new-tools:2025年の新ツール活用"
    "12:performance:パフォーマンス比較とROI"
    "13:summary:まとめ：2025年のゴールデンルール"
    "14:closing:おわりに"
)

# Process each section
for section_def in "${SECTIONS[@]}"; do
    IFS=':' read -r number dirname title <<< "${section_def}"

    section_dir="${CONTENTS_DIR}/${number}-${dirname}"
    mkdir -p "${section_dir}"

    echo "  📄 Creating section ${number}: ${title}"

    # Extract section content
    # Start from the section header, end at next ## or end of file
    awk -v title="^## ${title}" -v nextheader="^## " '
        $0 ~ title {found=1}
        found {
            if ($0 ~ nextheader && $0 !~ title) {exit}
            print
        }
    ' "${BLOG_FILE}" > "${section_dir}/blog.md"

    # Create README with metadata
    cat > "${section_dir}/README.md" <<EOF
# ${title}

**Section**: ${number}
**Directory**: ${dirname}
**Source**: blog_full.md

## Content

このディレクトリには「RustのDockerfile、2025年はこれでいこう」の第${number}章の内容が含まれています。

- \`blog.md\`: このセクションの本文

## Navigation

- Previous: Section $(printf "%02d" $((10#${number} - 1)))
- Next: Section $(printf "%02d" $((10#${number} + 1)))
- Full article: \`../blog_full.md\`
EOF
done

# Create main index
cat > "${CONTENTS_DIR}/README.md" <<'EOF'
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
EOF

echo ""
echo "✅ Successfully split blog into sections!"
echo "📁 Contents directory: ${CONTENTS_DIR}"
echo "📊 Created $(ls -d "${CONTENTS_DIR}"/*/ 2>/dev/null | wc -l | tr -d ' ') section directories"
echo ""
echo "Directory structure:"
tree "${CONTENTS_DIR}" -L 2 2>/dev/null || ls -R "${CONTENTS_DIR}"
