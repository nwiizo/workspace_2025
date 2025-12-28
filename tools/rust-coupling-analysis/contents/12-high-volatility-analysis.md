## High Volatility Analysis

No high volatility modules detected.
```

### テスト結果

実装したツールのテストは全て成功しています：

```bash
cargo test --all-targets
```

```
running 30 tests
test analyzer::tests::test_analyzer_creation ... ok
test balance::tests::test_balance_calculation ... ok
test balance::tests::test_balance_score_creation ... ok
test analyzer::tests::test_calculate_distance_same_level ... ok
test balance::tests::test_coupling_issue_creation ... ok
test analyzer::tests::test_calculate_distance_different_level ... ok
test balance::tests::test_identify_cascading_change ... ok
test balance::tests::test_identify_global_complexity ... ok
test balance::tests::test_modularity_ideal_close ... ok
test balance::tests::test_modularity_ideal_far ... ok
test balance::tests::test_modularity_poor ... ok
test balance::tests::test_no_issues_for_good_score ... ok
test metrics::tests::test_average_strength_calculation ... ok
test balance::tests::test_invalid_strength - should panic ... ok
test analyzer::tests::test_visit_inherent_impl ... ok
test analyzer::tests::test_function_signature_analysis ... ok
test analyzer::tests::test_visit_trait_impl ... ok
test metrics::tests::test_coupling_metrics_default ... ok
test metrics::tests::test_project_metrics_average ... ok
test report::tests::test_generate_empty_report ... ok
test report::tests::test_generate_summary ... ok
test report::tests::test_write_issues ... ok
test report::tests::test_write_strength_distribution ... ok
test volatility::tests::test_classify_volatility ... ok
test volatility::tests::test_get_change_count ... ok
test volatility::tests::test_volatility_analyzer_creation ... ok
test volatility::tests::test_top_volatile_files ... ok
test volatility::tests::test_volatility_level_to_score ... ok
test volatility::tests::test_volatility_score ... ok
test tests::test_analyze_project_with_current_project ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### プロジェクト構造

完成したプロジェクトの構造：

```
rust-coupling-analysis/
├── Cargo.toml
├── README.md
├── blog.md
├── split_blog.py
├── src/
│   ├── lib.rs              # メインライブラリ
│   ├── main.rs             # CLIエントリーポイント
│   ├── analyzer.rs         # AST解析（309行、11テスト）
│   ├── metrics.rs          # メトリクス（332行、3テスト）
│   ├── balance.rs          # バランススコア（376行、9テスト）
│   ├── volatility.rs       # 変動性分析（335行、7テスト）
│   └── report.rs           # レポート生成（385行、3テスト）
└── contents/               # blog.mdの分割版
    ├── 01-はじめに.md
    ├── 02-balancing-couplingの核心概念.md
    ├── 03-connascence共依存性.md
    ├── 04-rustにおける既存ツール.md
    ├── 05-balanced-couplingを測定するカスタムツールの実装.md
    ├── 06-実践的な使用例.md
    ├── 07-実践的なガイドライン.md
    ├── 08-おわりに.md
    └── README.md
```

### CLIオプション

実装したツールは以下のオプションをサポートしています：

```
USAGE:
    rust-coupling-analysis [OPTIONS] <PATH>

ARGS:
    <PATH>    Path to the Rust project to analyze (default: ./src)

OPTIONS:
    -h, --help           Print help information
    -o, --output <FILE>  Write report to file instead of stdout
    -s, --summary        Print condensed summary instead of full report
    -v, --version        Print version information
```

### コード品質

```bash
# フォーマットチェック
cargo fmt --check

# Lintチェック
cargo clippy -- -D warnings
```

全てのチェックが成功し、警告なしでコンパイルされます。
