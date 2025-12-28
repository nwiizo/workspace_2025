//! Chapter 11: Postgres as a Message Queue
//!
//! 書籍の11章で説明されているメッセージキュー機能。
//! - カスタムメッセージキューの実装
//! - FOR UPDATE SKIP LOCKED による並行処理
//! - LISTEN/NOTIFY によるリアルタイム通知
//! - トランザクショナルな一貫性

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// メッセージキューのデモ
///
/// Postgresをメッセージキューとして使用する方法を検証。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "メッセージキュー検証（Chapter 11）".bold());
    println!("{}", "-".repeat(50));

    // メッセージキュー用のスキーマとテーブルを作成
    println!("{}", "【メッセージキューのセットアップ】".yellow());

    client
        .execute("CREATE SCHEMA IF NOT EXISTS mq", &[])
        .await?;

    // ステータス型を作成
    client
        .execute(
            "DO $$
            BEGIN
                IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'mq_status') THEN
                    CREATE TYPE mq.mq_status AS ENUM ('new', 'processing', 'completed', 'failed');
                END IF;
            END $$",
            &[],
        )
        .await?;

    // キューテーブルを作成
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS mq.queue (
                id BIGSERIAL PRIMARY KEY,
                message JSON NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                status mq.mq_status NOT NULL DEFAULT 'new',
                processed_at TIMESTAMPTZ,
                worker_id TEXT
            )",
            &[],
        )
        .await?;

    // インデックスを作成（statusとcreated_atで効率的に検索）
    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_queue_status_created
             ON mq.queue (status, created_at)
             WHERE status = 'new'",
            &[],
        )
        .await?;

    println!("  {} mq.queue テーブルを作成", "✓".green());

    // enqueue関数を作成
    println!("\n{}", "【enqueue関数の作成】".yellow());

    client
        .execute(
            r#"
            CREATE OR REPLACE FUNCTION mq.enqueue(payload JSON)
            RETURNS BIGINT AS $$
            DECLARE
                new_id BIGINT;
            BEGIN
                INSERT INTO mq.queue (message)
                VALUES (payload)
                RETURNING id INTO new_id;

                -- NOTIFY で新しいメッセージを通知
                PERFORM pg_notify('queue_new_message', 'new_message');

                RETURN new_id;
            END;
            $$ LANGUAGE plpgsql
            "#,
            &[],
        )
        .await?;

    println!("  {} mq.enqueue() 関数を作成", "✓".green());

    // dequeue関数を作成（FOR UPDATE SKIP LOCKEDを使用）
    println!("\n{}", "【dequeue関数の作成】".yellow());
    println!(
        "{}",
        "  FOR UPDATE SKIP LOCKED で並行処理を実現".dimmed()
    );

    client
        .execute(
            r#"
            CREATE OR REPLACE FUNCTION mq.dequeue(
                worker TEXT,
                batch_size INT DEFAULT 1
            )
            RETURNS TABLE (
                message_id BIGINT,
                payload JSON
            ) AS $$
            BEGIN
                RETURN QUERY
                WITH locked_messages AS (
                    SELECT id
                    FROM mq.queue
                    WHERE status = 'new'
                    ORDER BY created_at
                    FOR UPDATE SKIP LOCKED
                    LIMIT batch_size
                ),
                updated AS (
                    UPDATE mq.queue q
                    SET status = 'processing',
                        worker_id = worker
                    FROM locked_messages lm
                    WHERE q.id = lm.id
                    RETURNING q.id, q.message
                )
                SELECT updated.id, updated.message
                FROM updated;
            END;
            $$ LANGUAGE plpgsql
            "#,
            &[],
        )
        .await?;

    println!("  {} mq.dequeue() 関数を作成", "✓".green());

    // complete関数を作成
    client
        .execute(
            r#"
            CREATE OR REPLACE FUNCTION mq.complete(message_id BIGINT)
            RETURNS VOID AS $$
            BEGIN
                UPDATE mq.queue
                SET status = 'completed',
                    processed_at = NOW()
                WHERE id = message_id;
            END;
            $$ LANGUAGE plpgsql
            "#,
            &[],
        )
        .await?;

    println!("  {} mq.complete() 関数を作成", "✓".green());

    // fail関数を作成
    client
        .execute(
            r#"
            CREATE OR REPLACE FUNCTION mq.fail(message_id BIGINT)
            RETURNS VOID AS $$
            BEGIN
                UPDATE mq.queue
                SET status = 'failed',
                    processed_at = NOW()
                WHERE id = message_id;
            END;
            $$ LANGUAGE plpgsql
            "#,
            &[],
        )
        .await?;

    println!("  {} mq.fail() 関数を作成", "✓".green());

    // 既存データをクリア
    client.execute("DELETE FROM mq.queue", &[]).await?;

    // メッセージをキューに追加
    println!("\n{}", "【メッセージのエンキュー】".yellow());

    let messages = vec![
        r#"{"service": "registration", "visitor": "Alice"}"#,
        r#"{"service": "license_renewal", "visitor": "Bob"}"#,
        r#"{"service": "registration", "visitor": "Charlie"}"#,
        r#"{"service": "id_card", "visitor": "Diana"}"#,
        r#"{"service": "license_renewal", "visitor": "Eve"}"#,
    ];

    for msg in &messages {
        let id = client
            .query_one(
                &format!("SELECT mq.enqueue('{}'::json) AS id", msg),
                &[],
            )
            .await?;
        let msg_id: i64 = id.get("id");
        println!("  メッセージ {} をエンキュー: {}", msg_id, msg.dimmed());
    }

    // キューの状態を確認
    println!("\n{}", "【キューの状態】".yellow());

    let queue_status = client
        .query(
            "SELECT id, message->>'visitor' AS visitor,
                    message->>'service' AS service, status::TEXT
             FROM mq.queue ORDER BY id",
            &[],
        )
        .await?;

    for row in &queue_status {
        let id: i64 = row.get("id");
        let visitor: String = row.get("visitor");
        let service: String = row.get("service");
        let status: String = row.get("status");
        println!(
            "  {} | {} | {} | {}",
            id,
            visitor.cyan(),
            service,
            status.yellow()
        );
    }

    // Worker 1がメッセージを取得
    println!("\n{}", "【Worker 1: メッセージを取得（2件）】".yellow());

    let worker1_messages = client
        .query("SELECT * FROM mq.dequeue('worker_1', 2)", &[])
        .await?;

    for row in &worker1_messages {
        let id: i64 = row.get("message_id");
        let payload: serde_json::Value = row.get("payload");
        println!(
            "  Worker 1 がメッセージ {} を取得: {}",
            id,
            payload.to_string().cyan()
        );
    }

    // Worker 2がメッセージを取得（SKIP LOCKEDで競合なし）
    println!(
        "\n{}",
        "【Worker 2: メッセージを取得（SKIP LOCKEDで競合回避）】".yellow()
    );

    let worker2_messages = client
        .query("SELECT * FROM mq.dequeue('worker_2', 2)", &[])
        .await?;

    for row in &worker2_messages {
        let id: i64 = row.get("message_id");
        let payload: serde_json::Value = row.get("payload");
        println!(
            "  Worker 2 がメッセージ {} を取得: {}",
            id,
            payload.to_string().cyan()
        );
    }

    // キューの状態を再確認
    println!("\n{}", "【キューの状態（処理中）】".yellow());

    let queue_status2 = client
        .query(
            "SELECT id, message->>'visitor' AS visitor, status::TEXT, worker_id
             FROM mq.queue ORDER BY id",
            &[],
        )
        .await?;

    for row in &queue_status2 {
        let id: i64 = row.get("id");
        let visitor: String = row.get("visitor");
        let status: String = row.get("status");
        let worker: Option<String> = row.get("worker_id");
        let status_colored = match status.as_str() {
            "new" => status.yellow(),
            "processing" => status.blue(),
            "completed" => status.green(),
            _ => status.red(),
        };
        println!(
            "  {} | {} | {} | {}",
            id,
            visitor.cyan(),
            status_colored,
            worker.unwrap_or_default().dimmed()
        );
    }

    // Worker 1が処理完了
    println!("\n{}", "【Worker 1: 処理完了】".yellow());

    for row in &worker1_messages {
        let id: i64 = row.get("message_id");
        client
            .execute(&format!("SELECT mq.complete({})", id), &[])
            .await?;
        println!("  メッセージ {} を完了", id);
    }

    // Worker 2が1件失敗、1件成功
    println!("\n{}", "【Worker 2: 1件成功、1件失敗】".yellow());

    if let Some(first) = worker2_messages.get(0) {
        let id: i64 = first.get("message_id");
        client
            .execute(&format!("SELECT mq.complete({})", id), &[])
            .await?;
        println!("  メッセージ {} を完了", id);
    }

    if let Some(second) = worker2_messages.get(1) {
        let id: i64 = second.get("message_id");
        client
            .execute(&format!("SELECT mq.fail({})", id), &[])
            .await?;
        println!("  メッセージ {} が失敗", id);
    }

    // 最終状態
    println!("\n{}", "【最終状態】".yellow());

    let final_status = client
        .query(
            "SELECT status::TEXT, COUNT(*) as count
             FROM mq.queue GROUP BY status ORDER BY status",
            &[],
        )
        .await?;

    for row in final_status {
        let status: String = row.get("status");
        let count: i64 = row.get("count");
        let status_colored = match status.as_str() {
            "new" => status.yellow(),
            "processing" => status.blue(),
            "completed" => status.green(),
            "failed" => status.red(),
            _ => status.normal(),
        };
        println!("  {}: {} 件", status_colored, count);
    }

    // LISTEN/NOTIFYの説明
    println!("\n{}", "【LISTEN/NOTIFY（リアルタイム通知）】".yellow());
    println!(
        "{}",
        "  ポーリング不要の非同期通知が可能".dimmed()
    );
    println!("");
    println!("  {}", "-- ワーカー側（リスナー）".dimmed());
    println!("  {}", "LISTEN queue_new_message;".cyan());
    println!("");
    println!("  {}", "-- プロデューサー側（エンキュー時に自動通知）".dimmed());
    println!(
        "  {}",
        "SELECT mq.enqueue('{\"visitor\": \"Frank\"}');".cyan()
    );
    println!(
        "  {}",
        "-- → pg_notify('queue_new_message', 'new_message')".dimmed()
    );

    // 制限事項
    println!("\n{}", "【LISTEN/NOTIFYの制限】".yellow());
    println!("  ⚠ 接続後の通知のみ受信可能（過去は受け取れない）");
    println!("  ⚠ プライマリノードへの接続が必要（レプリカ不可）");
    println!("  ⚠ 通知のペイロードサイズに制限あり（8000バイト）");

    // 使い分けの説明
    println!("\n{}", "【Postgresをメッセージキューとして使う基準】".yellow());
    println!("  {}:", "✓ 適切なケース".green());
    println!("    - トランザクショナルな一貫性が必要");
    println!("    - メッセージ量が比較的少ない");
    println!("    - 既にPostgresを使っている");

    println!("  {}:", "✗ 専用MQを検討すべきケース".red());
    println!("    - 超高スループットが必要");
    println!("    - 複雑なルーティングが必要");
    println!("    - Pub/Subパターンが中心");

    println!(
        "\n{}",
        "💡 ポイント: 「ビジネスロジック + メッセージ送信」をアトミックに実行できる。".dimmed()
    );
    println!(
        "{}",
        "   決済完了とメール送信キュー追加を1トランザクションで保証。".dimmed()
    );

    Ok(())
}
