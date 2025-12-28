## Connascence（共依存性）

結合度をさらに細かく分析するには、Meilir Page-Jones が提唱した「Connascence」の概念が役立つ。

### Static Connascence（静的共依存性）
コンパイル時に検出可能なもの。

- Connascence of Name（CoN）- 名前への依存
- Connascence of Type（CoT）- 型への依存
- Connascence of Meaning（CoM）- 値の意味への依存
- Connascence of Position（CoP）- パラメータ順序への依存
- Connascence of Algorithm（CoA）- アルゴリズムへの依存

### Dynamic Connascence（動的共依存性）
実行時に検出されるもの。

- Connascence of Execution（CoE）- 実行順序への依存
- Connascence of Timing（CoT）- タイミングへの依存
- Connascence of Value（CoV）- 値の同期的変更への依存
- Connascence of Identity（CoI）- 同一インスタンスへの依存

動的共依存性は、最も弱いものでも、最も強い静的共依存性よりも強い結合を意味する。
