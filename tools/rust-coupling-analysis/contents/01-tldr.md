## TL;DR

Vlad Khononovの「Balancing Coupling in Software Design」をRustで実装した結合度分析ツールを作りました。

- **3次元測定**: 結合の強さ・距離・変動性を数値化
- **自動検出**: 問題パターンを4種類自動識別
- **実行可能**: CLIツールとして実際に使える
- **テスト済**: 30個のユニットテストで検証済み

```bash
cargo run --release -- --summary
# → Files: 7, Modules: 7, Couplings: 0
```
