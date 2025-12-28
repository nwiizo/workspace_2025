## CLIオプション

```
USAGE:
    rust-coupling-analysis [OPTIONS] <PATH>

OPTIONS:
    -h, --help           ヘルプを表示
    -o, --output <FILE>  ファイルに出力
    -s, --summary        サマリーのみ表示
    -v, --version        バージョン情報

EXAMPLES:
    # 現在のプロジェクトを分析
    rust-coupling-analysis ./src

    # レポートをMarkdownで生成
    rust-coupling-analysis -o report.md ./src

    # サマリーのみ表示
    rust-coupling-analysis --summary ./src
```
