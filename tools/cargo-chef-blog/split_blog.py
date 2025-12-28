#!/usr/bin/env python3
"""
blog.mdをセクションごとに分割してcontents/配下に配置するスクリプト
"""

import os
import re
import shutil
from pathlib import Path

# セクションマッピング
SECTION_MAP = {
    "はじめに": "00-introduction",
    "RustのDockerビルドにおける根本的な問題": "01-problem",
    "Rustのビルドが特に問題になる理由": "02-why-rust",
    "技術スタックとアーキテクチャ": "03-architecture",
    "レシピコンセプトと動作原理": "04-recipe-concept",
    "動作原理と内部メカニズム": "05-internal-mechanism",
    "Docker統合とマルチステージビルド": "06-docker-integration",
    "ビルドプロセスの統合とサポート機能": "07-build-integration",
    "主要機能と実用的なユースケース": "08-use-cases",
    "パフォーマンス特性と測定結果": "09-performance",
    "実測検証: actix-webアプリケーションでの性能評価": "10-verification",
    "重要な制限と考慮事項": "11-limitations",
    "設計パターンとアーキテクチャの決定": "12-design-patterns",
    "代替アプローチとの比較": "13-alternatives",
    "エコシステムとリソース": "14-ecosystem",
    "まとめとベストプラクティス": "15-conclusion",
}

def main():
    blog_file = Path("blog.md")
    contents_dir = Path("contents")

    print("Starting blog.md splitting process...")

    # contentsディレクトリのクリーンアップと再作成
    if contents_dir.exists():
        print("Cleaning up old contents directory...")
        shutil.rmtree(contents_dir)

    contents_dir.mkdir(exist_ok=True)

    # blog.mdを読み込み
    with open(blog_file, "r", encoding="utf-8") as f:
        content = f.read()

    # セクションごとに分割
    # まず、導入部分を抽出（タイトルから最初の##まで）
    intro_match = re.search(r'^# cargo-chef.*?\n\n(.*?)(?=\n## )', content, re.DOTALL)
    if intro_match:
        intro_content = intro_match.group(1).strip()
        intro_dir = contents_dir / "00-introduction"
        intro_dir.mkdir(exist_ok=True)

        with open(intro_dir / "blog.md", "w", encoding="utf-8") as f:
            f.write(f"# はじめに\n\n{intro_content}\n")
        print(f"Created: {intro_dir}/blog.md")

    # 各セクションを抽出
    sections = re.findall(r'## (.+?)\n\n(.*?)(?=\n## |\Z)', content, re.DOTALL)

    for section_title, section_content in sections:
        section_title = section_title.strip()

        if section_title in SECTION_MAP:
            dir_name = SECTION_MAP[section_title]
            section_dir = contents_dir / dir_name
            section_dir.mkdir(exist_ok=True)

            # blog.mdを作成
            with open(section_dir / "blog.md", "w", encoding="utf-8") as f:
                f.write(f"# {section_title}\n\n{section_content.strip()}\n")
            print(f"Created: {section_dir}/blog.md")

            # README.mdを作成
            section_name_en = dir_name.split('-', 1)[1].replace('-', ' ').title()
            with open(section_dir / "README.md", "w", encoding="utf-8") as f:
                f.write(f"""# {section_title}

このディレクトリには、cargo-chefブログの「{section_title}」セクションが含まれています。

## ファイル

- `blog.md`: セクションの本文

## 構造

```
{dir_name}/
├── README.md  # このファイル
└── blog.md    # セクションの内容
```
""")
            print(f"Created: {section_dir}/README.md")

    # メインREADME.mdを作成
    with open(contents_dir / "README.md", "w", encoding="utf-8") as f:
        f.write("""# cargo-chef ブログコンテンツ

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
cat 00-introduction/blog.md \\
    01-problem/blog.md \\
    02-why-rust/blog.md \\
    03-architecture/blog.md \\
    04-recipe-concept/blog.md \\
    05-internal-mechanism/blog.md \\
    06-docker-integration/blog.md \\
    07-build-integration/blog.md \\
    08-use-cases/blog.md \\
    09-performance/blog.md \\
    10-verification/blog.md \\
    11-limitations/blog.md \\
    12-design-patterns/blog.md \\
    13-alternatives/blog.md \\
    14-ecosystem/blog.md \\
    15-conclusion/blog.md \\
    > ../blog-complete.md
```
""")
    print(f"Created: {contents_dir}/README.md")

    print("\n✓ Blog splitting completed successfully!\n")
    print("Structure created:")
    for section_file in sorted(contents_dir.glob("*/blog.md")):
        print(f"  {section_file}")

if __name__ == "__main__":
    main()
