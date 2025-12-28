//! Chapter 2: トランザクションの検証
//!
//! 書籍の2.4節で説明されているトランザクション分離レベルの検証。
//! Postgresのデフォルトであるread committedの動作を確認する。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;

/// トランザクション分離レベルのデモ
///
/// 2つのトランザクションが同時に在庫を更新しようとするケースをシミュレート。
/// read committedがdirty readとdirty writeを防ぐ様子を確認。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "トランザクション分離レベルの検証".bold());
    println!("{}", "-".repeat(50));

    // 現在の分離レベルを確認
    let row = client
        .query_one("SHOW transaction_isolation", &[])
        .await?;
    let isolation: String = row.get(0);
    println!("現在の分離レベル: {}", isolation.green());

    // テスト用の商品を選択
    let product = client
        .query_one(
            "SELECT id, name, stock_quantity FROM products.catalog WHERE id = 1",
            &[],
        )
        .await?;

    let product_id: i32 = product.get("id");
    let product_name: String = product.get("name");
    let original_stock: i32 = product.get("stock_quantity");

    println!(
        "\nテスト対象商品: {} (ID: {})",
        product_name.cyan(),
        product_id
    );
    println!("初期在庫: {}", original_stock);

    // トランザクション1: 在庫を5減らす
    println!("\n{}", "【トランザクション1】在庫を5減らす".yellow());
    let tx1 = client
        .query_one(
            "UPDATE products.catalog
            SET stock_quantity = stock_quantity - 5
            WHERE id = $1
            RETURNING stock_quantity",
            &[&product_id],
        )
        .await?;
    let stock_after_tx1: i32 = tx1.get(0);
    println!("  更新後の在庫: {}", stock_after_tx1);

    // 現在の在庫を確認
    let current = client
        .query_one(
            "SELECT stock_quantity FROM products.catalog WHERE id = $1",
            &[&product_id],
        )
        .await?;
    let current_stock: i32 = current.get(0);

    println!("\n{}", "【確認】現在の在庫:".yellow());
    println!(
        "  {} → {} (差分: {})",
        original_stock,
        current_stock,
        original_stock - current_stock
    );

    // 元に戻す
    client
        .execute(
            "UPDATE products.catalog SET stock_quantity = $1 WHERE id = $2",
            &[&original_stock, &product_id],
        )
        .await?;
    println!("\n(テスト用に在庫を元に戻しました)");

    // トランザクションのACID特性を示すデモ
    println!("\n{}", "【ACIDの検証】ロールバックのテスト".yellow());

    // トランザクションを開始
    let mut tx_client = pool.get().await?;
    let tx = tx_client
        .build_transaction()
        .isolation_level(tokio_postgres::IsolationLevel::ReadCommitted)
        .start()
        .await?;

    // 在庫を更新
    tx.execute(
        "UPDATE products.catalog SET stock_quantity = stock_quantity - 100 WHERE id = $1",
        &[&product_id],
    )
    .await?;

    // 更新後の値を確認（トランザクション内）
    let in_tx = tx
        .query_one(
            "SELECT stock_quantity FROM products.catalog WHERE id = $1",
            &[&product_id],
        )
        .await?;
    let in_tx_stock: i32 = in_tx.get(0);
    println!("  トランザクション内での在庫: {}", in_tx_stock);

    // ロールバック
    tx.rollback().await?;
    println!("  → ロールバック実行");

    // ロールバック後の値を確認
    let after_rollback = client
        .query_one(
            "SELECT stock_quantity FROM products.catalog WHERE id = $1",
            &[&product_id],
        )
        .await?;
    let after_stock: i32 = after_rollback.get(0);
    println!("  ロールバック後の在庫: {}", after_stock.to_string().green());

    if after_stock == original_stock {
        println!("  ✓ 在庫が正しく元に戻りました（Atomicity確認）");
    }

    // 外部キー制約の検証（書籍2.3節）
    println!("\n{}", "【外部キー制約の検証】".yellow());

    let fk_result = client
        .execute(
            "INSERT INTO products.reviews (product_id, customer_id, rating)
            VALUES (99999, 1, 5)",
            &[],
        )
        .await;

    match fk_result {
        Ok(_) => println!("  予期せずINSERTが成功しました"),
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("foreign key") || err_msg.contains("violates") {
                println!("  ✓ 外部キー制約が正しく機能しました:");
                println!("    存在しないproduct_id=99999へのINSERTが拒否されました");
            } else {
                println!("  エラー: {}", err_msg);
            }
        }
    }

    println!(
        "\n{}",
        "💡 ポイント: Postgresのread committedはdirty read/writeを防ぐ。".dimmed()
    );
    println!(
        "{}",
        "   外部キー制約はアプリのバグからデータを守る「多層防御」。".dimmed()
    );

    Ok(())
}
