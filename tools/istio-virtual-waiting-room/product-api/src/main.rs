use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use deadpool_postgres::{Config, Pool, Runtime};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::net::TcpListener;
use tokio_postgres::NoTls;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Product {
    id: i32,
    name: String,
    description: String,
    price: String,
    stock: i32,
}

#[derive(Clone)]
struct AppState {
    pool: Pool,
}

fn create_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some(env::var("POSTGRES_HOST").unwrap_or_else(|_| "postgres".to_string()));
    cfg.port = Some(
        env::var("POSTGRES_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .unwrap_or(5432),
    );
    cfg.dbname = Some(env::var("POSTGRES_DB").unwrap_or_else(|_| "shop".to_string()));
    cfg.user = Some(env::var("POSTGRES_USER").unwrap_or_else(|_| "shopuser".to_string()));
    cfg.password = Some(env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "shoppass".to_string()));

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create pool")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = create_pool();
    let state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let app = Router::new()
        .route("/api/products", get(list_products))
        .route("/api/products/:id/purchase", post(purchase_product))
        .route("/api/products/reset", post(reset_products))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state);

    info!("Product API is running on port {}", port);

    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn list_products(State(state): State<AppState>) -> impl IntoResponse {
    let client = match state.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<Product>::new()),
            );
        }
    };

    let rows = match client
        .query(
            "SELECT id, name, description, price, stock FROM products ORDER BY id",
            &[],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query products: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Vec::<Product>::new()),
            );
        }
    };

    let products: Vec<Product> = rows
        .iter()
        .map(|row| Product {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            price: row.get("price"),
            stock: row.get("stock"),
        })
        .collect();

    (StatusCode::OK, Json(products))
}

#[derive(Serialize)]
struct PurchaseResponse {
    success: bool,
    message: String,
    remaining_stock: i32,
}

async fn purchase_product(
    State(state): State<AppState>,
    Path(product_id): Path<i32>,
) -> impl IntoResponse {
    let client = match state.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PurchaseResponse {
                    success: false,
                    message: "データベース接続エラー".to_string(),
                    remaining_stock: 0,
                }),
            );
        }
    };

    // Use UPDATE with RETURNING to atomically decrement stock
    let result = client
        .query_opt(
            "UPDATE products SET stock = stock - 1
             WHERE id = $1 AND stock > 0
             RETURNING name, stock",
            &[&product_id],
        )
        .await;

    match result {
        Ok(Some(row)) => {
            let name: String = row.get("name");
            let remaining: i32 = row.get("stock");
            (
                StatusCode::OK,
                Json(PurchaseResponse {
                    success: true,
                    message: format!("{}を購入しました！", name),
                    remaining_stock: remaining,
                }),
            )
        }
        Ok(None) => {
            // Check if product exists
            let exists = client
                .query_opt("SELECT stock FROM products WHERE id = $1", &[&product_id])
                .await;

            match exists {
                Ok(Some(row)) => {
                    let stock: i32 = row.get("stock");
                    if stock == 0 {
                        (
                            StatusCode::CONFLICT,
                            Json(PurchaseResponse {
                                success: false,
                                message: "申し訳ありません。この商品は売り切れです。".to_string(),
                                remaining_stock: 0,
                            }),
                        )
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(PurchaseResponse {
                                success: false,
                                message: "購入処理に失敗しました。".to_string(),
                                remaining_stock: stock,
                            }),
                        )
                    }
                }
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(PurchaseResponse {
                        success: false,
                        message: "商品が見つかりません。".to_string(),
                        remaining_stock: 0,
                    }),
                ),
                Err(e) => {
                    error!("Failed to check product: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(PurchaseResponse {
                            success: false,
                            message: "データベースエラー".to_string(),
                            remaining_stock: 0,
                        }),
                    )
                }
            }
        }
        Err(e) => {
            error!("Failed to purchase: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(PurchaseResponse {
                    success: false,
                    message: "購入処理に失敗しました。".to_string(),
                    remaining_stock: 0,
                }),
            )
        }
    }
}

async fn reset_products(State(state): State<AppState>) -> impl IntoResponse {
    let client = match state.pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to get DB connection: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error");
        }
    };

    let result = client
        .execute(
            "UPDATE products SET stock = CASE id
                WHEN 1 THEN 3
                WHEN 2 THEN 5
                WHEN 3 THEN 2
                WHEN 4 THEN 4
                WHEN 5 THEN 1
             END",
            &[],
        )
        .await;

    match result {
        Ok(_) => (StatusCode::OK, "Products reset"),
        Err(e) => {
            error!("Failed to reset products: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Reset failed")
        }
    }
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Check database connectivity
    match state.pool.get().await {
        Ok(client) => match client.query_one("SELECT 1", &[]).await {
            Ok(_) => (StatusCode::OK, "Healthy"),
            Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "Database unhealthy"),
        },
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "Database unavailable"),
    }
}
