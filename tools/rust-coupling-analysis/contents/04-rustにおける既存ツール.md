## Rustにおける既存ツール

### 1. cargo-modules

モジュール構造と依存関係を可視化できる。

**インストール:**
```bash
cargo install cargo-modules
```

**使い方:**
```bash
# モジュール構造をツリー表示
cargo modules structure

# モジュール間の依存関係をグラフ表示
cargo modules dependencies

# 循環依存の検出
cargo modules dependencies --acyclic

# 孤立したファイルの検出
cargo modules orphans
```

**特徴:**
- モジュール階層の視覚化
- 循環依存の検出（リファクタリングの重要な手がかり）
- 未使用ファイルの発見
- GraphViz と連携可能

### 2. rust-code-analysis

Mozilla が開発した、多言語対応のコードメトリクス計測ツール。

**インストール:**
```bash
cargo install rust-code-analysis-cli
```

**使い方:**
```bash
# 単一ファイルの分析
rust-code-analysis-cli --metrics -p src/main.rs

# プロジェクト全体の分析
rust-code-analysis-cli --metrics -p ./src

# JSON形式で出力
rust-code-analysis-cli --metrics -O json -o metrics.json -p ./src
```

**計測可能なメトリクス**

- CC (Cyclomatic Complexity) - 循環的複雑度
- COGNITIVE - 認知的複雑度
- HALSTEAD - Halstead メトリクス（Bugs, Difficulty, Effort, Volume 等）
- LOC 系 - SLOC、PLOC、LLOC 等
- NOM - メソッド数
- NARGS - 引数の数
- NEXITS - 出口の数
- WMC - クラスごとの循環的複雑度の合計

Rust コードの複雑度を他言語と比較する研究でも使用されており、信頼性が高い。

### 3. cargo tree

Cargo 組み込みの依存関係ツリー表示コマンド。

**使い方:**
```bash
# 依存関係ツリーの表示
cargo tree

# 深さを制限
cargo tree --depth 1

# 特定のパッケージを除外
cargo tree --prune serde

# 重複する依存関係を表示
cargo tree --duplicates

# リバース依存関係（何がこのクレートに依存しているか）
cargo tree --invert <package-name>
```

### 4. その他の有用なツール

- cargo-deps - GraphViz DOT ファイルを生成
- cargo-depgraph - 視覚的な依存関係グラフ
- tokei - コード統計（行数、言語別集計）
- cargo-deny - 依存関係のポリシー検証
