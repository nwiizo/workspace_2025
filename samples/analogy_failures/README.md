# analogy_failures

ブログ記事「おい、類推するな」のサンプルコード集。

類推による誤解を実際のRustコードで示します。

## サンプル一覧

### 1. ownership - 所有権の類推失敗

「本の貸し借り」の類推でRustの所有権を理解しようとして、借用チェッカーに怒られる例。

```bash
cargo run --bin ownership
```

### 2. async_mutex - 非同期処理のMutex直列化

「料理を並行して作る」つもりが、Mutexで直列実行になる例。

```bash
cargo run --bin async_mutex
```

出力例:
```
[メインコンロ] パスタ の調理を開始
[メインコンロ] パスタ の調理が完了
[メインコンロ] ソース の調理を開始
[メインコンロ] ソース の調理が完了

合計調理時間: 4.00秒
```

期待は2秒だが、実際は4秒かかる。

### 3. async_deadlock - デッドロック

2つのタスクが互いのリソースを待ち合ってデッドロックする例。

```bash
cargo run --bin async_deadlock
```

出力例:
```
[タスク1] コンロを確保しました！
[タスク2] オーブンを確保しました！
[タスク1] オーブンを確保しようとしています...
[タスク2] コンロを確保しようとしています...

⚠️  タイムアウト！デッドロックが発生しました。
```

### 4. cache_race - キャッシュの競合状態

読み取りと更新が同時に発生して、古いデータがキャッシュに残る例。

```bash
cargo run --bin cache_race
```

出力例:
```
DBの値:        email=Some("new@example.com")
キャッシュの値: email=Some("old@example.com")

⚠️  キャッシュに古いデータが残っている！
```

## DB連携サンプル（Docker必要）

`transaction_demo/` にPostgreSQLを使った検証があります。

```bash
cd transaction_demo
docker compose up -d
cargo run --bin transaction      # トランザクションの類推失敗
cargo run --bin cache_race_db    # キャッシュ競合（DB版）
docker compose down
```

詳細は `transaction_demo/README.md` を参照。

## 関連ブログ記事

- `../../blogs/blog.md` - 「おい、類推するな」
