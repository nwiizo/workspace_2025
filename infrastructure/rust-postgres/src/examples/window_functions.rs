//! Chapter 3: Window Functionsの検証
//!
//! 書籍の3.3-3.4節で説明されているWindow Functionsの検証。
//! GROUP BYとは異なり、元の行を保持したまま集計値を計算できる。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct RankedProduct {
    name: String,
    category: String,
    price: String,
    category_rank: i64,
}

#[derive(Tabled)]
struct RunningTotal {
    customer: String,
    product: String,
    amount: String,
    running_total: String,
}

/// Window Functionsのデモ
///
/// ROW_NUMBER, RANK, SUM() OVERなどの
/// Window Functionsを実際に使ってみる。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "Window Functionsの検証".bold());
    println!("{}", "-".repeat(50));

    // ROW_NUMBER / RANK
    println!("{}", "【ROW_NUMBER】カテゴリ内でのランキング".yellow());

    let rows = client
        .query(
            "SELECT
                name,
                category,
                price,
                ROW_NUMBER() OVER (
                    PARTITION BY category
                    ORDER BY price DESC
                ) AS category_rank
            FROM products.catalog
            ORDER BY category, category_rank",
            &[],
        )
        .await?;

    let products: Vec<RankedProduct> = rows
        .iter()
        .map(|row| RankedProduct {
            name: row.get("name"),
            category: row.get("category"),
            price: format!("${}", row.get::<_, Decimal>("price")),
            category_rank: row.get("category_rank"),
        })
        .collect();

    println!("{}", Table::new(&products));

    // 累計（Running Total）
    println!("\n{}", "【SUM OVER】注文の累計金額".yellow());

    let running = client
        .query(
            "SELECT
                c.name AS customer,
                p.name AS product,
                (oi.quantity * oi.price) AS amount,
                SUM(oi.quantity * oi.price) OVER (
                    PARTITION BY o.customer_id
                    ORDER BY o.completed_at
                    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
                ) AS running_total
            FROM sales.orders o
            JOIN customers.accounts c ON c.id = o.customer_id
            JOIN sales.order_items oi ON oi.order_id = o.id
            JOIN products.catalog p ON p.id = oi.product_id
            WHERE o.status = 'completed'
            ORDER BY o.customer_id, o.completed_at
            LIMIT 10",
            &[],
        )
        .await?;

    let totals: Vec<RunningTotal> = running
        .iter()
        .map(|row| RunningTotal {
            customer: row.get("customer"),
            product: row.get("product"),
            amount: format!("${:.2}", row.get::<_, Decimal>("amount")),
            running_total: format!("${:.2}", row.get::<_, Decimal>("running_total")),
        })
        .collect();

    if totals.is_empty() {
        println!("  (完了済み注文がありません)");
    } else {
        println!("{}", Table::new(&totals));
    }

    // LAG / LEAD
    println!("\n{}", "【LAG】前回との比較".yellow());

    let lag_rows = client
        .query(
            "WITH daily_plays AS (
                SELECT
                    DATE(play_start_time) AS play_date,
                    COUNT(*) AS play_count
                FROM streaming.plays
                GROUP BY DATE(play_start_time)
            )
            SELECT
                play_date,
                play_count,
                LAG(play_count, 1) OVER (ORDER BY play_date) AS prev_day_count,
                play_count - LAG(play_count, 1) OVER (ORDER BY play_date) AS diff
            FROM daily_plays
            ORDER BY play_date DESC
            LIMIT 7",
            &[],
        )
        .await?;

    println!("  日別再生数と前日比:");
    for row in lag_rows {
        let date: chrono::NaiveDate = row.get("play_date");
        let count: i64 = row.get("play_count");
        let prev: Option<i64> = row.get("prev_day_count");
        let diff: Option<i64> = row.get("diff");

        let diff_str = match diff {
            Some(d) if d > 0 => format!("+{}", d).green().to_string(),
            Some(d) if d < 0 => format!("{}", d).red().to_string(),
            Some(_) => "±0".to_string(),
            None => "-".to_string(),
        };

        println!(
            "    {} - {} 再生 (前日: {}, 差分: {})",
            date,
            count,
            prev.map(|p| p.to_string()).unwrap_or("-".to_string()),
            diff_str
        );
    }

    // NTILE
    println!("\n{}", "【NTILE】顧客を購入額で4分位に分類".yellow());

    let ntile_rows = client
        .query(
            "WITH customer_totals AS (
                SELECT
                    c.id,
                    c.name,
                    COALESCE(SUM(o.total_amount), 0) AS total_spent
                FROM customers.accounts c
                LEFT JOIN sales.orders o ON o.customer_id = c.id AND o.status = 'completed'
                GROUP BY c.id
            )
            SELECT
                name,
                total_spent,
                NTILE(4) OVER (ORDER BY total_spent DESC) AS quartile
            FROM customer_totals
            WHERE total_spent > 0
            ORDER BY total_spent DESC
            LIMIT 12",
            &[],
        )
        .await?;

    println!("  顧客の購入額による四分位:");
    for row in ntile_rows {
        let name: String = row.get("name");
        let spent: Decimal = row.get("total_spent");
        let quartile: i32 = row.get("quartile");

        let tier = match quartile {
            1 => "上位25%".green(),
            2 => "上位50%".cyan(),
            3 => "下位50%".yellow(),
            4 => "下位25%".red(),
            _ => "?".normal(),
        };

        println!("    {} - ${:.2} ({})", name, spent, tier);
    }

    println!(
        "\n{}",
        "💡 ポイント: Window FunctionsはGROUP BYと違い、".dimmed()
    );
    println!(
        "{}",
        "   元の行を保持したまま集計値を計算できる。".dimmed()
    );
    println!(
        "{}",
        "   LAG/LEADで前後の行との比較、NTILEで分位分類も簡単。".dimmed()
    );

    Ok(())
}
