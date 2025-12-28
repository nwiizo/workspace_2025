//! Chapter 2: データベース関数の検証
//!
//! 書籍の2.6節で説明されているPL/pgSQL関数の検証。
//! カート操作をデータベース関数として実装し、
//! アプリとDB間の往復を減らす。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use tabled::{Table, Tabled};
use uuid::Uuid;

#[derive(Tabled)]
struct CartItem {
    order_id: String,
    product_name: String,
    quantity: i32,
    unit_price: String,
    line_total: String,
}

/// カート操作のデモ
///
/// データベース関数`sales.order_add_item`を使って、
/// 1回のクエリでカートに商品を追加する。
///
/// ## データベース関数を使う利点
///
/// 1. アプリ↔DB間の往復を減らせる
/// 2. トランザクションが自動的に保証される
/// 3. 複数クライアントで同じロジックを共有できる
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "データベース関数によるカート操作".bold());
    println!("{}", "-".repeat(50));

    // テスト用のカスタマーIDを取得
    let customer = client
        .query_one(
            "SELECT id, name FROM customers.accounts ORDER BY id LIMIT 1",
            &[],
        )
        .await?;
    let customer_id: i32 = customer.get("id");
    let customer_name: String = customer.get("name");

    println!("顧客: {} (ID: {})", customer_name.cyan(), customer_id);

    // 既存のpending orderがあれば削除（クリーンな状態で開始）
    client
        .execute(
            "DELETE FROM sales.orders WHERE customer_id = $1 AND status = 'pending'",
            &[&customer_id],
        )
        .await?;

    println!("\n{}", "【ステップ1】カートに商品を追加".yellow());

    // 商品1を追加
    let cart1 = client
        .query(
            "SELECT * FROM sales.order_add_item($1, 1, 2)",
            &[&customer_id],
        )
        .await?;

    print_cart_items(&cart1, "Espresso x 2 を追加");

    // 商品2を追加
    println!("\n{}", "【ステップ2】さらに商品を追加".yellow());
    let cart2 = client
        .query(
            "SELECT * FROM sales.order_add_item($1, 2, 1)",
            &[&customer_id],
        )
        .await?;

    print_cart_items(&cart2, "Cappuccino x 1 を追加");

    // 同じ商品を追加（数量が加算される）
    println!("\n{}", "【ステップ3】同じ商品をさらに追加".yellow());
    let cart3 = client
        .query(
            "SELECT * FROM sales.order_add_item($1, 1, 3)",
            &[&customer_id],
        )
        .await?;

    print_cart_items(&cart3, "Espresso x 3 を追加（既存の2に加算）");

    // チェックアウト
    println!("\n{}", "【ステップ4】チェックアウト".yellow());
    let checkout = client
        .query_one(
            "SELECT * FROM sales.order_checkout($1)",
            &[&customer_id],
        )
        .await?;

    let order_id: Uuid = checkout.get("out_order_id");
    let total: Decimal = checkout.get("out_total_amount");
    let status: String = checkout.get("out_status");

    println!("  注文ID: {}", order_id.to_string()[..8].cyan());
    println!("  合計金額: ${}", total.to_string().green());
    println!("  ステータス: {}", status.green());

    // 在庫の変動を確認
    println!("\n{}", "【確認】在庫の変動".yellow());
    let stock = client
        .query(
            "SELECT id, name, stock_quantity
            FROM products.catalog
            WHERE id IN (1, 2)
            ORDER BY id",
            &[],
        )
        .await?;

    for row in stock {
        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let qty: i32 = row.get("stock_quantity");
        println!("  {} (ID: {}): 在庫 {}", name, id, qty);
    }

    // トリガーの動作確認
    println!("\n{}", "【トリガーの検証】".yellow());
    println!("  order_itemsの変更でorders.total_amountが自動更新されることを確認");

    // 別の注文を作成
    let new_order = client
        .query(
            "SELECT * FROM sales.order_add_item($1, 3, 1)",
            &[&customer_id],
        )
        .await?;

    if !new_order.is_empty() {
        let order_id: Uuid = new_order[0].get("out_order_id");

        // 合計を直接確認
        let order_total = client
            .query_one(
                "SELECT total_amount FROM sales.orders WHERE id = $1",
                &[&order_id],
            )
            .await?;
        let total: Decimal = order_total.get(0);
        println!(
            "  新規注文の合計: ${} (トリガーで自動計算)",
            total.to_string().green()
        );
    }

    println!(
        "\n{}",
        "💡 ポイント: データベース関数で複数操作を1回のクエリに。".dimmed()
    );
    println!(
        "{}",
        "   トリガーで派生データを自動更新。ただし使いすぎに注意。".dimmed()
    );

    Ok(())
}

fn print_cart_items(rows: &[tokio_postgres::Row], message: &str) {
    println!("  {}", message.dimmed());

    let items: Vec<CartItem> = rows
        .iter()
        .map(|row| CartItem {
            order_id: row.get::<_, Uuid>("out_order_id").to_string()[..8].to_string(),
            product_name: row.get("out_product_name"),
            quantity: row.get("out_quantity"),
            unit_price: format!("${}", row.get::<_, Decimal>("out_unit_price")),
            line_total: format!("${}", row.get::<_, Decimal>("out_line_total")),
        })
        .collect();

    println!("{}", Table::new(&items));
}
