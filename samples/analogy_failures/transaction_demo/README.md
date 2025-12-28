# transaction_demo

ブログ記事「おい、類推するな」のDB連携サンプルコード。

実際のPostgreSQLを使って、トランザクションとキャッシュの類推失敗を検証します。

## 前提条件

- Docker
- Rust

## セットアップ

```bash
# PostgreSQLを起動
docker compose up -d

# 起動を確認
docker compose ps
```

## サンプル

### 1. トランザクションの類推失敗

「銀行の振込みたいなもの」という類推では見えない問題を検証。

```bash
cargo run --bin transaction
```

出力例:
```
--- シナリオ2: 外部API成功後にDB更新が失敗 ---
（Alice → frozen_account: 5,000円 - 受取人アカウント凍結で失敗）
  → 通知を送信しました（外部API呼び出し）
送金失敗: 受取人のアカウントが凍結されています
残高:
  alice: Alice (90000円)
  bob: Bob (60000円)
送金通知: 2件
  alice → bob: 10000円
  alice → frozen_account: 5000円

⚠️  問題発生！
   - DBの残高は変わっていない（ロールバックされた）
   - しかし通知は送信された！（外部APIは取り消せない）
```

**ポイント**: 外部API（通知）はトランザクションの外で実行されるため、ロールバックしても取り消せない。

### 2. キャッシュ競合の類推失敗

「手元に辞書を置く」という類推では見えない問題を検証。

```bash
cargo run --bin cache_race_db
```

出力例:
```
[T2] リクエストA: DBから取得中...
  → 取得完了: email = "old@example.com"
  (処理に時間がかかっている...)
[T3] リクエストB: DBを更新
  → 更新完了: email = new@example.com
[T4] リクエストB: キャッシュを無効化
  → 無効化完了
[T5] リクエストA: キャッシュに保存
  → 保存完了: email = "old@example.com" ← 古いデータ！

--- 結果確認 ---
DB: email = Some("new@example.com")
キャッシュ: email = Some("old@example.com")

⚠️  不整合発生！
```

**ポイント**: 読み取りと更新が同時に発生すると、古いデータがキャッシュに残る競合状態が起きる。

## クリーンアップ

```bash
docker compose down
```

## 関連

- `../` - メモリ内のみで動作するサンプル
- `../../blogs/blog.md` - ブログ記事「おい、類推するな」
