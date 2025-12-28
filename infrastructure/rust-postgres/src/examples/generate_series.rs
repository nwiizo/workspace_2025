//! Chapter 1: generate_seriesを使ったモックデータ生成
//!
//! 書籍の1.5節で紹介されている`generate_series`関数の検証。
//! 外部ツールを使わずに、SQLだけで複雑なモックデータを生成できる。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use rust_decimal::Decimal;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Trade {
    id: i64,
    buyer_id: i32,
    symbol: String,
    order_quantity: i32,
    bid_price: Decimal,
}

/// generate_seriesを使ってモックデータを生成するデモ
///
/// 書籍の例では株取引データを生成していたが、
/// ここでは同様のパターンでランダムな取引データを生成する。
pub async fn run(pool: &Pool) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "generate_seriesによるモックデータ生成".bold());
    println!("{}", "-".repeat(50));

    // 一時テーブルを作成してgenerate_seriesでデータを生成
    client
        .execute(
            "CREATE TEMP TABLE IF NOT EXISTS temp_trades (
                id BIGINT,
                buyer_id INT,
                symbol VARCHAR(10),
                order_quantity INT,
                bid_price DECIMAL(10,2)
            )",
            &[],
        )
        .await?;

    // generate_seriesでデータを生成
    let inserted = client
        .execute(
            "INSERT INTO temp_trades (id, buyer_id, symbol, order_quantity, bid_price)
            SELECT
                id,
                floor(random() * 10 + 1)::int as buyer_id,
                (ARRAY['AAPL','GOOGL','MSFT','AMZN','META'])[floor(random() * 5 + 1)::int] as symbol,
                floor(random() * 100 + 1)::int as order_quantity,
                round((random() * 500 + 100)::numeric, 2) as bid_price
            FROM generate_series(1, 20) AS id",
            &[],
        )
        .await?;

    println!("生成されたレコード数: {}", inserted.to_string().green());

    // 生成されたデータを表示
    let rows = client
        .query(
            "SELECT id, buyer_id, symbol, order_quantity, bid_price
            FROM temp_trades
            ORDER BY id
            LIMIT 10",
            &[],
        )
        .await?;

    let trades: Vec<Trade> = rows
        .iter()
        .map(|row| Trade {
            id: row.get("id"),
            buyer_id: row.get("buyer_id"),
            symbol: row.get("symbol"),
            order_quantity: row.get("order_quantity"),
            bid_price: row.get("bid_price"),
        })
        .collect();

    println!("\n生成されたデータ（先頭10件）:");
    println!("{}", Table::new(&trades));

    // 集計クエリ（書籍の1.6節に相当）
    println!("\n{}", "シンボル別の取引量集計:".bold());

    let summary = client
        .query(
            "SELECT symbol, COUNT(*) as total_trades, SUM(order_quantity) as total_quantity
            FROM temp_trades
            GROUP BY symbol
            ORDER BY total_quantity DESC",
            &[],
        )
        .await?;

    for row in summary {
        let symbol: String = row.get("symbol");
        let total_trades: i64 = row.get("total_trades");
        let total_quantity: i64 = row.get("total_quantity");
        println!(
            "  {} - 取引数: {}, 合計数量: {}",
            symbol.cyan(),
            total_trades,
            total_quantity
        );
    }

    // 一時テーブルをクリーンアップ
    client.execute("DROP TABLE temp_trades", &[]).await?;

    println!(
        "\n{}",
        "💡 ポイント: generate_seriesとrandom()の組み合わせで、".dimmed()
    );
    println!(
        "{}",
        "   外部ツールなしでモックデータを生成できる".dimmed()
    );

    Ok(())
}
