# レシピコンセプトと動作原理

「レシピ」は、ソースコードなしで依存関係をビルドするために必要な最小限の情報を捉えた**JSONファイル**(recipe.json)です。これは Python の requirements.txt と同じ目的を果たしますが、Rust のより複雑なプロジェクト構造に対応しています。

**レシピの内容:**

- プロジェクト全体のすべての Cargo.toml ファイルとその相対パス
- Cargo.lock ファイル(存在する場合)、正確な依存関係バージョンのため
- すべてのバイナリとライブラリの明示的な宣言—正規の場所(src/main.rs、src/lib.rs)にあるものでも
- スケルトン再構築のためのプロジェクト構造メタデータ

```rust
pub struct Recipe {
    pub skeleton: Skeleton,
}

pub struct Skeleton {
    manifests: Vec<Manifest>,
    lock_file: Option<String>,
}
```

この構造は人間が読める JSON にシリアライズされ、レシピはデバッグ可能で検査可能です。明示的なターゲット宣言により、Cargo が通常ファイルの場所からターゲットを推測する場合でも、信頼性の高いキャッシュが保証されます。
