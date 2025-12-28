//! Chapter 2: Viewの検証
//!
//! 書籍の2.7節で説明されているView（ビュー）の検証。
//! Viewは「名前付きクエリ」として、複雑なJOINを隠蔽できる。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ProductSales {
    product_name: String,
    category: String,
    total_sold: String,
    revenue: String,
}

/// Viewのデモ
///
/// 事前に作成した`sales.product_sales_summary`ビューを使って、
/// 商品ごとの売上サマリーを取得する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "Viewによる複雑なクエリの抽象化".bold());
    println!("{}", "-".repeat(50));

    // Viewの定義を確認
    println!("{}", "【Viewの定義】".yellow());
    let view_def = client
        .query_one(
            "SELECT pg_get_viewdef('sales.product_sales_summary'::regclass, true)",
            &[],
        )
        .await?;
    let definition: String = view_def.get(0);
    println!("{}", definition.dimmed());

    // Viewを使ったクエリ
    println!("\n{}", "【Viewからデータを取得】".yellow());
    let rows = client
        .query(
            "SELECT product_name, category, total_quantity_sold, total_revenue
            FROM sales.product_sales_summary
            WHERE total_quantity_sold > 0
            ORDER BY total_revenue DESC
            LIMIT 10",
            &[],
        )
        .await?;

    let sales: Vec<ProductSales> = rows
        .iter()
        .map(|row| ProductSales {
            product_name: row.get("product_name"),
            category: row.get("category"),
            total_sold: row.get::<_, i64>("total_quantity_sold").to_string(),
            revenue: format!("${:.2}", row.get::<_, Decimal>("total_revenue")),
        })
        .collect();

    if sales.is_empty() {
        println!("  (売上データがありません)");
    } else {
        println!("{}", Table::new(&sales));
    }

    // カテゴリでフィルタ
    println!("\n{}", "【カテゴリでフィルタ】".yellow());
    let coffee = client
        .query(
            "SELECT product_name, total_quantity_sold, total_revenue
            FROM sales.product_sales_summary
            WHERE category = 'coffee'
            ORDER BY total_quantity_sold DESC",
            &[],
        )
        .await?;

    println!("  コーヒーカテゴリの売上:");
    for row in coffee {
        let name: String = row.get("product_name");
        let qty: i64 = row.get("total_quantity_sold");
        let revenue: Decimal = row.get("total_revenue");
        println!("    {} - {} 杯, ${}", name.cyan(), qty, revenue);
    }

    // Materialized Viewの作成と比較
    println!("\n{}", "【Materialized Viewの検証】".yellow());

    // 一時的にMaterialized Viewを作成
    client
        .execute(
            "CREATE MATERIALIZED VIEW IF NOT EXISTS sales.product_sales_snapshot AS
            SELECT * FROM sales.product_sales_summary",
            &[],
        )
        .await?;

    // 実行計画の比較
    println!("  通常のViewの実行計画:");
    let explain_view = client
        .query(
            "EXPLAIN (FORMAT TEXT)
            SELECT * FROM sales.product_sales_summary WHERE category = 'coffee'",
            &[],
        )
        .await?;
    for row in explain_view.iter().take(3) {
        let plan: String = row.get(0);
        println!("    {}", plan.dimmed());
    }

    println!("\n  Materialized Viewの実行計画:");
    let explain_mat = client
        .query(
            "EXPLAIN (FORMAT TEXT)
            SELECT * FROM sales.product_sales_snapshot WHERE category = 'coffee'",
            &[],
        )
        .await?;
    for row in explain_mat.iter().take(3) {
        let plan: String = row.get(0);
        println!("    {}", plan.dimmed());
    }

    // クリーンアップ
    client
        .execute("DROP MATERIALIZED VIEW IF EXISTS sales.product_sales_snapshot", &[])
        .await?;

    println!(
        "\n{}",
        "💡 ポイント: 通常ViewはクエリのたびにJOINを実行。".dimmed()
    );
    println!(
        "{}",
        "   Materialized Viewは結果をキャッシュ。リフレッシュタイミングが重要。".dimmed()
    );

    Ok(())
}
