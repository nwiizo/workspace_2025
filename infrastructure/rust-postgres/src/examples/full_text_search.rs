//! 全文検索の検証
//!
//! PostgreSQLの全文検索機能は、Elasticsearchの代替となりうる。
//! 「Just Use Postgres」思想の典型例の一つ。

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::Pool;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct SearchResult {
    name: String,
    category: String,
    description: String,
    rank: String,
}

/// 全文検索のデモ
///
/// products.catalogのdescriptionカラムに対して
/// 全文検索を実行する。GINインデックスを使った効率的な検索。
pub async fn run(pool: &Pool, query: &str) -> Result<()> {
    let client = pool.get().await?;

    println!("{}", "PostgreSQL全文検索の検証".bold());
    println!("{}", "-".repeat(50));
    println!("検索クエリ: {}", query.cyan());

    // 基本的な全文検索
    println!("\n{}", "【基本的な全文検索】".yellow());

    let rows = client
        .query(
            "SELECT
                name,
                category,
                LEFT(description, 60) || '...' AS description,
                ts_rank(
                    to_tsvector('english', COALESCE(description, '')),
                    plainto_tsquery('english', $1)
                ) AS rank
            FROM products.catalog
            WHERE to_tsvector('english', COALESCE(description, ''))
                @@ plainto_tsquery('english', $1)
            ORDER BY rank DESC",
            &[&query],
        )
        .await?;

    let results: Vec<SearchResult> = rows
        .iter()
        .map(|row| SearchResult {
            name: row.get("name"),
            category: row.get("category"),
            description: row.get("description"),
            rank: format!("{:.4}", row.get::<_, f32>("rank")),
        })
        .collect();

    if results.is_empty() {
        println!("  検索結果がありません");
    } else {
        println!("{}", Table::new(&results));
    }

    // tsqueryの構文
    println!("\n{}", "【tsqueryの構文】".yellow());

    // AND検索
    println!("  AND検索 (coffee & strong):");
    let and_results = client
        .query(
            "SELECT name, description
            FROM products.catalog
            WHERE to_tsvector('english', COALESCE(description, ''))
                @@ to_tsquery('english', 'coffee & strong')
            LIMIT 3",
            &[],
        )
        .await?;
    for row in and_results {
        let name: String = row.get("name");
        println!("    - {}", name.cyan());
    }

    // OR検索
    println!("\n  OR検索 (chocolate | sweet):");
    let or_results = client
        .query(
            "SELECT name, description
            FROM products.catalog
            WHERE to_tsvector('english', COALESCE(description, ''))
                @@ to_tsquery('english', 'chocolate | sweet')
            LIMIT 3",
            &[],
        )
        .await?;
    for row in or_results {
        let name: String = row.get("name");
        println!("    - {}", name.cyan());
    }

    // NOT検索
    println!("\n  NOT検索 (coffee & !milk):");
    let not_results = client
        .query(
            "SELECT name, description
            FROM products.catalog
            WHERE to_tsvector('english', COALESCE(description, ''))
                @@ to_tsquery('english', 'coffee & !milk')
            LIMIT 3",
            &[],
        )
        .await?;
    for row in not_results {
        let name: String = row.get("name");
        println!("    - {}", name.cyan());
    }

    // ハイライト表示
    println!("\n{}", "【ts_headlineによるハイライト】".yellow());

    let highlighted = client
        .query(
            "SELECT
                name,
                ts_headline('english',
                    description,
                    plainto_tsquery('english', $1),
                    'StartSel=<<, StopSel=>>, MaxWords=35, MinWords=15'
                ) AS highlighted
            FROM products.catalog
            WHERE to_tsvector('english', COALESCE(description, ''))
                @@ plainto_tsquery('english', $1)
            LIMIT 3",
            &[&query],
        )
        .await?;

    for row in highlighted {
        let name: String = row.get("name");
        let hl: String = row.get("highlighted");
        println!("  {}:", name.cyan());
        // << >> を色付きに変換（シンプルに表示）
        let colored_hl = hl.replace("<<", "\x1b[33m").replace(">>", "\x1b[0m");
        println!("    {}", colored_hl);
    }

    // 複数カラムの検索
    println!("\n{}", "【複数カラムの検索（名前 + 説明）】".yellow());

    let multi_col = client
        .query(
            "SELECT
                name,
                category,
                ts_rank(
                    setweight(to_tsvector('english', name), 'A') ||
                    setweight(to_tsvector('english', COALESCE(description, '')), 'B'),
                    plainto_tsquery('english', $1)
                ) AS rank
            FROM products.catalog
            WHERE
                setweight(to_tsvector('english', name), 'A') ||
                setweight(to_tsvector('english', COALESCE(description, '')), 'B')
                @@ plainto_tsquery('english', $1)
            ORDER BY rank DESC
            LIMIT 5",
            &[&query],
        )
        .await?;

    println!("  重み付け検索結果（名前: A, 説明: B）:");
    for row in multi_col {
        let name: String = row.get("name");
        let rank: f32 = row.get("rank");
        println!("    {} (rank: {:.4})", name.cyan(), rank);
    }

    // 辞書とステミング
    println!("\n{}", "【辞書とステミング】".yellow());

    let stemming = client
        .query_one(
            "SELECT
                to_tsvector('english', 'running runs runner')::TEXT AS tsvector,
                plainto_tsquery('english', 'run')::TEXT AS tsquery",
            &[],
        )
        .await?;

    let tsvector: String = stemming.get(0);
    let tsquery: String = stemming.get(1);
    println!("  'running runs runner' → {}", tsvector.dimmed());
    println!("  'run' → {}", tsquery.dimmed());
    println!(
        "  → すべて 'run' にステミングされるのでマッチする"
    );

    // インデックスの確認
    println!("\n{}", "【GINインデックスの確認】".yellow());

    let indexes = client
        .query(
            "SELECT indexname, indexdef
            FROM pg_indexes
            WHERE tablename = 'catalog' AND schemaname = 'products'
              AND indexdef LIKE '%gin%'",
            &[],
        )
        .await?;

    if indexes.is_empty() {
        println!("  GINインデックスがありません");
    } else {
        for row in indexes {
            let name: String = row.get("indexname");
            let def: String = row.get("indexdef");
            println!("  {}: {}", name.cyan(), def.dimmed());
        }
    }

    println!(
        "\n{}",
        "💡 ポイント: PostgreSQLの全文検索はElasticsearchの代替に。".dimmed()
    );
    println!(
        "{}",
        "   GINインデックスで高速検索、ts_headlineでハイライト。".dimmed()
    );
    println!(
        "{}",
        "   setweightで複数カラムに重み付けも可能。".dimmed()
    );

    Ok(())
}
