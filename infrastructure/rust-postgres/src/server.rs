//! Axum API サーバー
//!
//! PostgreSQL の高度な機能を活用した REST API サーバーを起動します。
//!
//! ## 起動方法
//!
//! ```bash
//! cargo run --bin api-server
//! ```
//!
//! デフォルトで `http://localhost:3001` で起動します。
//!
//! ## 環境変数（Google OAuth 用）
//!
//! - `GOOGLE_CLIENT_ID` - Google OAuth クライアント ID
//! - `GOOGLE_CLIENT_SECRET` - Google OAuth クライアントシークレット
//! - `OAUTH_REDIRECT_URL` - OAuth コールバック URL（デフォルト: http://localhost:3001/api/auth/google/callback）
//! - `FRONTEND_URL` - フロントエンド URL（デフォルト: http://localhost:3000）

use anyhow::Result;
use colored::Colorize;
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

mod api;
mod examples;

use api::{create_router, AppState, OAuthConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // バナー表示
    println!("\n{}", "=".repeat(60).cyan());
    println!(
        "{}",
        "  Rust + PostgreSQL - API Server".cyan().bold()
    );
    println!("{}", "=".repeat(60).cyan());

    // データベース接続プールを作成
    println!("\n{}", "Connecting to database...".yellow());
    let pool = create_pool().await?;

    // 接続テスト
    let client = pool.get().await?;
    client.query_one("SELECT 1", &[]).await?;
    println!("{}", "  Database connection: OK".green());

    // AppState を作成
    let mut state = AppState::new(pool);

    // OAuth 設定を読み込む（オプション）
    match OAuthConfig::from_env() {
        Ok(oauth_config) => {
            println!("{}", "  Google OAuth: Configured".green());
            state = state.with_oauth(oauth_config);
        }
        Err(_) => {
            println!("{}", "  Google OAuth: Not configured (set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET)".yellow());
        }
    }

    // ルーターを作成
    let app = create_router(state);

    // サーバーを起動
    let addr = "0.0.0.0:3001";
    println!("\n{}", format!("Starting server on http://{}", addr).green().bold());
    println!("\n{}", "Available endpoints:".yellow());
    println!("  GET  /api/health              - Health check");
    println!("  GET  /api/auth/google/login   - Start Google OAuth login");
    println!("  GET  /api/auth/status         - Check authentication status");
    println!("  POST /api/auth/logout         - Logout");
    println!("  GET  /api/products/search     - Search products (Full-Text + JSONB)");
    println!("  GET  /api/products/categories - List categories");
    println!("  GET  /api/products/{{id}}       - Get product detail");
    println!("  GET  /api/products/{{id}}/similar - Similar products (pgvector)");
    println!("  GET  /api/metrics             - Get metrics (TimescaleDB)");
    println!("  GET  /api/sensors             - List sensors");
    println!("  GET  /api/stores              - List all stores");
    println!("  GET  /api/stores/nearby       - Nearby stores (PostGIS)");
    println!("\n{}", "=".repeat(60).cyan());

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_pool() -> Result<Pool> {
    let mut cfg = Config::new();
    cfg.host = Some(
        std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string()),
    );
    cfg.port = Some(
        std::env::var("PGPORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5432),
    );
    cfg.user = Some(
        std::env::var("PGUSER").unwrap_or_else(|_| "postgres".to_string()),
    );
    cfg.password = Some(
        std::env::var("PGPASSWORD").unwrap_or_else(|_| "password".to_string()),
    );
    cfg.dbname = Some(
        std::env::var("PGDATABASE").unwrap_or_else(|_| "coffee_chain".to_string()),
    );
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    Ok(pool)
}
