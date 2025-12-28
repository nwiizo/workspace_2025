// トランザクションの類推失敗: 外部APIとロールバックの問題
//
// 「銀行の振込みたいなもの」という類推では見えない問題を実際のDBで検証

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::sync::atomic::{AtomicUsize, Ordering};

// 送金通知のカウンター（外部APIの代わり）
static NOTIFICATION_COUNT: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== トランザクションの類推失敗: 外部APIとロールバックの問題 ===\n");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://demo:demo@localhost:5433/analogy_demo")
        .await?;

    // 初期状態を表示
    println!("--- 初期状態 ---");
    show_balances(&pool).await?;

    // シナリオ1: 正常な送金
    println!("\n--- シナリオ1: 正常な送金（Alice → Bob: 10,000円）---");
    match transfer_money_naive(&pool, "alice", "bob", 10000).await {
        Ok(_) => println!("送金成功！"),
        Err(e) => println!("送金失敗: {}", e),
    }
    show_balances(&pool).await?;
    show_notifications(&pool).await?;

    // シナリオ2: 外部API成功後にDB更新が失敗
    println!("\n--- シナリオ2: 外部API成功後にDB更新が失敗 ---");
    println!("（Alice → frozen_account: 5,000円 - 受取人アカウント凍結で失敗）");
    match transfer_money_naive(&pool, "alice", "frozen_account", 5000).await {
        Ok(_) => println!("送金成功！"),
        Err(e) => println!("送金失敗: {}", e),
    }
    show_balances(&pool).await?;
    show_notifications(&pool).await?;

    println!("\n⚠️  問題発生！");
    println!("   - DBの残高は変わっていない（ロールバックされた）");
    println!("   - しかし通知は送信された！（外部APIは取り消せない）");
    println!("   - 「銀行の振込」の類推では、この問題は見えない");

    // シナリオ3: 正しいアプローチ
    println!("\n--- シナリオ3: 正しいアプローチ（通知をトランザクション後に送る）---");

    // データをリセット
    reset_data(&pool).await?;
    println!("（データをリセット）");
    show_balances(&pool).await?;

    println!("\n正しいアプローチで送金（Alice → Bob: 10,000円）:");
    match transfer_money_correct(&pool, "alice", "bob", 10000).await {
        Ok(_) => println!("送金成功！"),
        Err(e) => println!("送金失敗: {}", e),
    }
    show_balances(&pool).await?;
    show_notifications(&pool).await?;

    println!("\n正しいアプローチで残高不足送金（Bob → Alice: 1,000,000円）:");
    match transfer_money_correct(&pool, "bob", "alice", 1_000_000).await {
        Ok(_) => println!("送金成功！"),
        Err(e) => println!("送金失敗: {}", e),
    }
    show_balances(&pool).await?;
    show_notifications(&pool).await?;

    println!("\n✅ 正しいアプローチでは:");
    println!("   - トランザクションがコミットされてから通知を送る");
    println!("   - 失敗した場合は通知も送られない");

    Ok(())
}

/// 問題のあるアプローチ: トランザクション内で外部APIを呼ぶ
async fn transfer_money_naive(
    pool: &PgPool,
    from: &str,
    to: &str,
    amount: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    // 送金元の残高チェック（先にチェック）
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM accounts WHERE user_id = $1")
        .bind(from)
        .fetch_one(&mut *tx)
        .await?;

    if balance < amount {
        return Err("残高不足".into());
    }

    // 送金元の残高を減らす
    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE user_id = $2")
        .bind(amount)
        .bind(from)
        .execute(&mut *tx)
        .await?;

    // ここで「外部API」を呼ぶ（問題！）
    // 実際のシステムではメール送信やSlack通知など
    // この通知はトランザクションの外で実行される
    send_notification(pool, from, to, amount).await?;
    println!("  → 通知を送信しました（外部API呼び出し）");

    // 送金先の残高を増やす（これが失敗する可能性がある）
    // 何らかの理由で失敗をシミュレート（例: 受取人のアカウントが凍結）
    if to == "frozen_account" {
        return Err("受取人のアカウントが凍結されています".into());
    }

    sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE user_id = $2")
        .bind(amount)
        .bind(to)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

/// 正しいアプローチ: トランザクションがコミットされてから通知を送る
async fn transfer_money_correct(
    pool: &PgPool,
    from: &str,
    to: &str,
    amount: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tx = pool.begin().await?;

    // 送金元の残高を減らす
    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE user_id = $2")
        .bind(amount)
        .bind(from)
        .execute(&mut *tx)
        .await?;

    // 送金先の残高を増やす
    sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE user_id = $2")
        .bind(amount)
        .bind(to)
        .execute(&mut *tx)
        .await?;

    // 送金元の残高チェック
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM accounts WHERE user_id = $1")
        .bind(from)
        .fetch_one(&mut *tx)
        .await?;

    if balance < 0 {
        // ロールバックされる（txがドロップされる）
        return Err("残高不足".into());
    }

    // まずトランザクションをコミット
    tx.commit().await?;

    // コミット成功後に通知を送る（外部APIはトランザクションの外）
    send_notification(pool, from, to, amount).await?;
    println!("  → 通知を送信しました（コミット後）");

    Ok(())
}

/// 外部API呼び出しをシミュレート
async fn send_notification(
    pool: &PgPool,
    from: &str,
    to: &str,
    amount: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    // 実際のシステムでは、ここでメール送信やSlack通知などを行う
    // この例では、DBに記録してカウントする

    NOTIFICATION_COUNT.fetch_add(1, Ordering::SeqCst);

    sqlx::query(
        "INSERT INTO transfer_notifications (from_user, to_user, amount) VALUES ($1, $2, $3)",
    )
    .bind(from)
    .bind(to)
    .bind(amount)
    .execute(pool)
    .await?;

    Ok(())
}

async fn show_balances(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let rows = sqlx::query("SELECT user_id, name, balance FROM accounts ORDER BY user_id")
        .fetch_all(pool)
        .await?;

    println!("残高:");
    for row in rows {
        let user_id: &str = row.get("user_id");
        let name: &str = row.get("name");
        let balance: i64 = row.get("balance");
        println!("  {}: {} ({}円)", user_id, name, balance);
    }
    Ok(())
}

async fn show_notifications(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transfer_notifications")
        .fetch_one(pool)
        .await?;

    println!("送金通知: {}件", count);

    if count > 0 {
        let rows = sqlx::query(
            "SELECT from_user, to_user, amount, created_at FROM transfer_notifications ORDER BY id",
        )
        .fetch_all(pool)
        .await?;

        for row in rows {
            let from: &str = row.get("from_user");
            let to: &str = row.get("to_user");
            let amount: i64 = row.get("amount");
            println!("  {} → {}: {}円", from, to, amount);
        }
    }
    Ok(())
}

async fn reset_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DELETE FROM transfer_notifications")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE accounts SET balance = 100000 WHERE user_id = 'alice'")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE accounts SET balance = 50000 WHERE user_id = 'bob'")
        .execute(pool)
        .await?;
    Ok(())
}
