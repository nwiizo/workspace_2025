//! Chapter 2: スキーマ構造の確認
//!
//! 書籍の2.1節で説明されているDatabase→Schema→Tableの三層構造を確認する。
//! マイクロサービス的にスキーマを分離する設計パターンの実例。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct SchemaInfo {
    schema_name: String,
    table_name: String,
    row_count: i64,
}

/// スキーマ構造を表示するデモ
///
/// coffee_chainデータベースの構造:
/// - products: 商品カタログとレビュー
/// - customers: 顧客アカウント
/// - sales: 注文と注文明細
/// - streaming: 音楽再生履歴（Chapter 3用）
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "データベースのスキーマ構造".bold());
    println!("{}", "-".repeat(50));

    // 各スキーマとテーブルの一覧を取得
    let rows = client
        .query(
            "SELECT
                n.nspname AS schema_name,
                c.relname AS table_name,
                COALESCE(s.n_live_tup, 0) AS row_count
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
            WHERE n.nspname IN ('products', 'customers', 'sales', 'streaming')
              AND c.relkind = 'r'
            ORDER BY n.nspname, c.relname",
            &[],
        )
        .await?;

    let schema_infos: Vec<SchemaInfo> = rows
        .iter()
        .map(|row| SchemaInfo {
            schema_name: row.get("schema_name"),
            table_name: row.get("table_name"),
            row_count: row.get("row_count"),
        })
        .collect();

    println!("{}", Table::new(&schema_infos));

    // スキーマ間のリレーションを表示
    println!("\n{}", "スキーマ間の外部キー関係:".bold());

    let fk_rows = client
        .query(
            "SELECT
                tc.table_schema AS from_schema,
                tc.table_name AS from_table,
                kcu.column_name AS from_column,
                ccu.table_schema AS to_schema,
                ccu.table_name AS to_table,
                ccu.column_name AS to_column
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage ccu
                ON ccu.constraint_name = tc.constraint_name
            WHERE tc.constraint_type = 'FOREIGN KEY'
              AND tc.table_schema IN ('products', 'customers', 'sales', 'streaming')
            ORDER BY tc.table_schema, tc.table_name",
            &[],
        )
        .await?;

    for row in fk_rows {
        let from_schema: String = row.get("from_schema");
        let from_table: String = row.get("from_table");
        let from_column: String = row.get("from_column");
        let to_schema: String = row.get("to_schema");
        let to_table: String = row.get("to_table");
        let to_column: String = row.get("to_column");

        println!(
            "  {}.{}.{} → {}.{}.{}",
            from_schema.cyan(),
            from_table,
            from_column,
            to_schema.green(),
            to_table,
            to_column
        );
    }

    // クロススキーマJOINの例
    println!("\n{}", "クロススキーマJOINの例:".bold());
    println!("{}", "（注文と商品情報を結合）".dimmed());

    let cross_join = client
        .query(
            "SELECT
                c.name AS customer_name,
                p.name AS product_name,
                oi.quantity,
                oi.price,
                (oi.quantity * oi.price) AS line_total
            FROM sales.orders o
            JOIN customers.accounts c ON c.id = o.customer_id
            JOIN sales.order_items oi ON oi.order_id = o.id
            JOIN products.catalog p ON p.id = oi.product_id
            WHERE o.status = 'completed'
            ORDER BY o.completed_at DESC
            LIMIT 5",
            &[],
        )
        .await?;

    for row in cross_join {
        let customer: String = row.get("customer_name");
        let product: String = row.get("product_name");
        let qty: i32 = row.get("quantity");
        let price: rust_decimal::Decimal = row.get("price");
        let total: rust_decimal::Decimal = row.get("line_total");
        println!(
            "  {} が {} を {} 個購入 (単価: ${}, 小計: ${})",
            customer.cyan(),
            product.green(),
            qty,
            price,
            total
        );
    }

    println!(
        "\n{}",
        "💡 ポイント: スキーマを分けることで、マイクロサービス的な".dimmed()
    );
    println!(
        "{}",
        "   モジュール分離ができる。同じDB内なのでJOINも可能。".dimmed()
    );

    Ok(())
}
