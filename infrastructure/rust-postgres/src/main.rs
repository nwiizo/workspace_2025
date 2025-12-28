//! Just Use Postgres! - 技術検証プログラム
//!
//! 「Just Use Postgres!」書籍で紹介されている各機能をRustで検証します。
//!
//! # 検証内容
//!
//! - Chapter 1: 基本操作（generate_series、基本クエリ）
//! - Chapter 2: RDBMS機能（スキーマ、制約、トランザクション、関数、トリガー、View）
//! - Chapter 3: Modern SQL（CTE、Window Functions、Recursive Queries）
//! - Chapter 4: Indexes（B-tree、Hash、Covering、Partial、Expression）
//! - Chapter 5: JSON（演算子、JSON Path、GINインデックス）
//! - Chapter 6: Full-Text Search（tsvector、tsquery、GIN/GiST）
//! - Chapter 7: Extensions（pgcrypto、pg_trgm、uuid-ossp）
//! - Chapter 8: Generative AI（pgvector、ベクトル検索）
//! - Chapter 9: Time Series（TimescaleDB、Hypertable、time_bucket）
//! - Chapter 10: Geospatial Data（PostGIS、geometry/geography、ST_Distance）
//! - Chapter 11: Message Queue（FOR UPDATE SKIP LOCKED、LISTEN/NOTIFY）

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

mod examples;

/// Just Use Postgres! 技術検証CLI
#[derive(Parser)]
#[command(name = "just-use-postgres")]
#[command(about = "Technical verification for 'Just Use Postgres!' book examples")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Chapter 1: generate_seriesを使ったデータ生成
    GenerateSeries,
    /// Chapter 2: スキーマ構造の確認
    ShowSchemas,
    /// Chapter 2: トランザクションの検証
    Transaction,
    /// Chapter 2: データベース関数の検証（カート操作）
    CartDemo,
    /// Chapter 2: Viewの検証
    ViewDemo,
    /// Chapter 3: CTEの検証
    CteDemo,
    /// Chapter 3: Window Functionsの検証
    WindowFunctions,
    /// Chapter 3: Recursive Queryの検証（連続再生履歴）
    RecursiveQuery,
    /// Chapter 4: インデックス戦略の検証
    IndexDemo,
    /// Chapter 5: JSON機能の詳細検証
    JsonAdvanced,
    /// Chapter 5: JSONB基本機能の検証
    JsonbDemo,
    /// Chapter 6: 全文検索の詳細検証
    FullTextAdvanced,
    /// Chapter 6: 全文検索の検証
    FullTextSearch {
        /// 検索クエリ
        query: String,
    },
    /// Chapter 7: 拡張機能の検証
    Extensions,
    /// Chapter 8: pgvectorによるベクトル検索
    Pgvector,
    /// Chapter 9: TimescaleDBによる時系列データ
    Timescaledb,
    /// Chapter 10: PostGISによる地理空間データ
    Postgis,
    /// Chapter 11: メッセージキューの検証
    MessageQueue,
    /// すべてのデモを実行
    All,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let pool = create_pool().await?;

    match cli.command {
        Commands::GenerateSeries => examples::generate_series::run(&pool).await?,
        Commands::ShowSchemas => examples::schemas::run(&pool).await?,
        Commands::Transaction => examples::transaction::run(&pool).await?,
        Commands::CartDemo => examples::cart::run(&pool).await?,
        Commands::ViewDemo => examples::view::run(&pool).await?,
        Commands::CteDemo => examples::cte::run(&pool).await?,
        Commands::WindowFunctions => examples::window_functions::run(&pool).await?,
        Commands::RecursiveQuery => examples::recursive::run(&pool).await?,
        Commands::IndexDemo => examples::indexes::run(&pool).await?,
        Commands::JsonAdvanced => examples::json_advanced::run(&pool).await?,
        Commands::JsonbDemo => examples::jsonb::run(&pool).await?,
        Commands::FullTextAdvanced => examples::full_text_advanced::run(&pool).await?,
        Commands::FullTextSearch { query } => examples::full_text_search::run(&pool, &query).await?,
        Commands::Extensions => examples::extensions::run(&pool).await?,
        Commands::Pgvector => examples::pgvector::run(&pool).await?,
        Commands::Timescaledb => examples::timescaledb::run(&pool).await?,
        Commands::Postgis => examples::postgis::run(&pool).await?,
        Commands::MessageQueue => examples::message_queue::run(&pool).await?,
        Commands::All => {
            println!("\n{}", "=".repeat(80).cyan());
            println!("{}", "Just Use Postgres! - 全機能デモ".cyan().bold());
            println!("{}\n", "=".repeat(80).cyan());

            println!("\n{}", "【Chapter 1: generate_series】".yellow().bold());
            examples::generate_series::run(&pool).await?;

            println!("\n{}", "【Chapter 2: スキーマ構造】".yellow().bold());
            examples::schemas::run(&pool).await?;

            println!("\n{}", "【Chapter 2: トランザクション】".yellow().bold());
            examples::transaction::run(&pool).await?;

            println!("\n{}", "【Chapter 2: カート操作（DB関数）】".yellow().bold());
            examples::cart::run(&pool).await?;

            println!("\n{}", "【Chapter 2: View】".yellow().bold());
            examples::view::run(&pool).await?;

            println!("\n{}", "【Chapter 3: CTE】".yellow().bold());
            examples::cte::run(&pool).await?;

            println!("\n{}", "【Chapter 3: Window Functions】".yellow().bold());
            examples::window_functions::run(&pool).await?;

            println!("\n{}", "【Chapter 3: Recursive Query】".yellow().bold());
            examples::recursive::run(&pool).await?;

            println!("\n{}", "【Chapter 4: インデックス戦略】".yellow().bold());
            examples::indexes::run(&pool).await?;

            println!("\n{}", "【Chapter 5: JSONB基本】".yellow().bold());
            examples::jsonb::run(&pool).await?;

            println!("\n{}", "【Chapter 5: JSON詳細】".yellow().bold());
            examples::json_advanced::run(&pool).await?;

            println!("\n{}", "【Chapter 6: 全文検索基本】".yellow().bold());
            examples::full_text_search::run(&pool, "coffee espresso").await?;

            println!("\n{}", "【Chapter 6: 全文検索詳細】".yellow().bold());
            examples::full_text_advanced::run(&pool).await?;

            println!("\n{}", "【Chapter 7: 拡張機能】".yellow().bold());
            examples::extensions::run(&pool).await?;

            println!("\n{}", "【Chapter 8: pgvector（ベクトル検索）】".yellow().bold());
            examples::pgvector::run(&pool).await?;

            println!("\n{}", "【Chapter 9: TimescaleDB（時系列データ）】".yellow().bold());
            examples::timescaledb::run(&pool).await?;

            println!("\n{}", "【Chapter 10: PostGIS（地理空間データ）】".yellow().bold());
            examples::postgis::run(&pool).await?;

            println!("\n{}", "【Chapter 11: メッセージキュー】".yellow().bold());
            examples::message_queue::run(&pool).await?;

            println!("\n{}", "=".repeat(80).green());
            println!("{}", "すべてのデモが完了しました！".green().bold());
            println!("{}\n", "=".repeat(80).green());
        }
    }

    Ok(())
}

async fn create_pool() -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("password".to_string());
    cfg.dbname = Some("coffee_chain".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    Ok(pool)
}
