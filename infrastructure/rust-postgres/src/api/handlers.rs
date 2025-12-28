//! HTTP ハンドラ
//!
//! 各エンドポイントの処理を実装します。

use axum::{
    extract::{Path, Query, State},
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::{AppError, AppState};

// ============================================================================
// 商品関連
// ============================================================================

/// 商品
#[derive(Debug, Serialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub price: Decimal,
    pub description: Option<String>,
    pub attributes: Option<JsonValue>,
    pub rank: Option<f32>,
}

/// 商品検索パラメータ
#[derive(Debug, Deserialize)]
pub struct ProductSearchParams {
    pub q: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
}

/// 商品検索結果
#[derive(Debug, Serialize)]
pub struct ProductSearchResult {
    pub items: Vec<Product>,
    pub total: i64,
}

/// 商品検索（全文検索 + JSONB フィルタリング）
pub async fn search_products(
    State(state): State<AppState>,
    Query(params): Query<ProductSearchParams>,
) -> Result<Json<ProductSearchResult>, AppError> {
    let client = state.db.get().await?;

    // 動的SQLの構築（実運用ではprepared statementを推奨）
    let rows = client
        .query(
            r#"
            SELECT
                id, name, category, price, description, attributes,
                CASE
                    WHEN $1::TEXT IS NOT NULL THEN
                        COALESCE(ts_rank(search_vector, plainto_tsquery('english', $1)), 0.5)::REAL
                    ELSE NULL
                END as rank
            FROM products.catalog
            WHERE ($1::TEXT IS NULL OR name ILIKE '%' || $1 || '%' OR description ILIKE '%' || $1 || '%')
              AND ($2::TEXT IS NULL OR category = $2)
              AND ($3::NUMERIC IS NULL OR price >= $3)
              AND ($4::NUMERIC IS NULL OR price <= $4)
            ORDER BY
                CASE WHEN $1::TEXT IS NOT NULL THEN ts_rank(search_vector, plainto_tsquery('english', $1)) ELSE 0 END DESC,
                name
            LIMIT 50
            "#,
            &[
                &params.q,
                &params.category,
                &params.min_price.map(|p| Decimal::from_f64_retain(p).unwrap_or_default()),
                &params.max_price.map(|p| Decimal::from_f64_retain(p).unwrap_or_default()),
            ],
        )
        .await?;

    let items: Vec<Product> = rows
        .iter()
        .map(|row| Product {
            id: row.get("id"),
            name: row.get("name"),
            category: row.get("category"),
            price: row.get("price"),
            description: row.get("description"),
            attributes: row.get("attributes"),
            rank: row.get("rank"),
        })
        .collect();

    let total = items.len() as i64;

    Ok(Json(ProductSearchResult { items, total }))
}

/// 商品詳細取得
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Product>, AppError> {
    let client = state.db.get().await?;

    let row = client
        .query_opt(
            "SELECT id, name, category, price, description, attributes FROM products.catalog WHERE id = $1",
            &[&id],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Product {} not found", id)))?;

    Ok(Json(Product {
        id: row.get("id"),
        name: row.get("name"),
        category: row.get("category"),
        price: row.get("price"),
        description: row.get("description"),
        attributes: row.get("attributes"),
        rank: None,
    }))
}

/// 類似商品取得（pgvector用 - モック実装）
#[derive(Debug, Serialize)]
pub struct SimilarProduct {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub price: Decimal,
    pub similarity: f32,
}

#[derive(Debug, Deserialize)]
pub struct SimilarParams {
    pub limit: Option<i32>,
}

pub async fn get_similar_products(
    State(state): State<AppState>,
    Path(product_id): Path<i32>,
    Query(params): Query<SimilarParams>,
) -> Result<Json<Vec<SimilarProduct>>, AppError> {
    let client = state.db.get().await?;
    let limit = params.limit.unwrap_or(5).min(20);

    // pgvector がセットアップされていない場合は同カテゴリの商品を返す
    let rows = client
        .query(
            r#"
            SELECT p2.id, p2.name, p2.category, p2.price,
                   0.8::REAL as similarity
            FROM products.catalog p1
            JOIN products.catalog p2 ON p1.category = p2.category AND p1.id != p2.id
            WHERE p1.id = $1
            ORDER BY p2.price DESC
            LIMIT $2
            "#,
            &[&product_id, &(limit as i64)],
        )
        .await?;

    if rows.is_empty() {
        return Err(AppError::NotFound(format!(
            "Product {} not found",
            product_id
        )));
    }

    let products: Vec<SimilarProduct> = rows
        .iter()
        .map(|row| SimilarProduct {
            id: row.get("id"),
            name: row.get("name"),
            category: row.get("category"),
            price: row.get("price"),
            similarity: row.get("similarity"),
        })
        .collect();

    Ok(Json(products))
}

/// カテゴリ一覧取得
pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, AppError> {
    let client = state.db.get().await?;

    let rows = client
        .query(
            "SELECT DISTINCT category FROM products.catalog ORDER BY category",
            &[],
        )
        .await?;

    let categories: Vec<String> = rows.iter().map(|row| row.get("category")).collect();

    Ok(Json(categories))
}

// ============================================================================
// メトリクス関連（TimescaleDB）
// ============================================================================

/// メトリクスバケット
#[derive(Debug, Serialize)]
pub struct MetricsBucket {
    pub time: String,
    #[serde(rename = "avgValue")]
    pub avg_value: f64,
    #[serde(rename = "minValue")]
    pub min_value: f64,
    #[serde(rename = "maxValue")]
    pub max_value: f64,
    pub count: i64,
}

/// メトリクスクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub sensor_id: i32,
    pub start: String,
    pub end: String,
    pub interval: String,
}

/// メトリクス取得（TimescaleDB time_bucket）
pub async fn get_metrics(
    State(state): State<AppState>,
    Query(params): Query<MetricsQuery>,
) -> Result<Json<Vec<MetricsBucket>>, AppError> {
    // インターバルのバリデーション
    let valid_intervals = ["1 minute", "5 minutes", "1 hour", "1 day"];
    if !valid_intervals.contains(&params.interval.as_str()) {
        return Err(AppError::InvalidInput(format!(
            "Invalid interval '{}'. Valid values: {:?}",
            params.interval, valid_intervals
        )));
    }

    let client = state.db.get().await?;

    // TimescaleDB のスキーマが存在する場合のクエリ
    // 存在しない場合はモックデータを返す
    let check_table = client
        .query_opt(
            "SELECT 1 FROM information_schema.tables
             WHERE table_schema = 'watch' AND table_name = 'heart_rate_measurements'",
            &[],
        )
        .await?;

    if check_table.is_some() {
        let query = format!(
            r#"
            SELECT
                time_bucket('{}', recorded_at) AS time,
                AVG(heart_rate)::FLOAT8 AS avg_value,
                MIN(heart_rate)::FLOAT8 AS min_value,
                MAX(heart_rate)::FLOAT8 AS max_value,
                COUNT(*)::BIGINT AS count
            FROM watch.heart_rate_measurements
            WHERE watch_id = $1
              AND recorded_at >= $2::TIMESTAMPTZ
              AND recorded_at < $3::TIMESTAMPTZ
            GROUP BY time
            ORDER BY time
            "#,
            params.interval
        );

        let rows = client
            .query(&query, &[&params.sensor_id, &params.start, &params.end])
            .await?;

        let metrics: Vec<MetricsBucket> = rows
            .iter()
            .map(|row| {
                let time: chrono::DateTime<chrono::Utc> = row.get("time");
                MetricsBucket {
                    time: time.to_rfc3339(),
                    avg_value: row.get("avg_value"),
                    min_value: row.get("min_value"),
                    max_value: row.get("max_value"),
                    count: row.get("count"),
                }
            })
            .collect();

        Ok(Json(metrics))
    } else {
        // モックデータを返す
        Ok(Json(generate_mock_metrics()))
    }
}

fn generate_mock_metrics() -> Vec<MetricsBucket> {
    use chrono::{Duration, Utc};

    let now = Utc::now();
    (0..24)
        .map(|i| {
            let time = now - Duration::hours(24 - i);
            MetricsBucket {
                time: time.to_rfc3339(),
                avg_value: 70.0 + (i as f64 * 0.5),
                min_value: 60.0 + (i as f64 * 0.3),
                max_value: 80.0 + (i as f64 * 0.7),
                count: 60,
            }
        })
        .collect()
}

/// センサー
#[derive(Debug, Serialize)]
pub struct Sensor {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub sensor_type: String,
    pub location: String,
}

/// センサー一覧取得
pub async fn get_sensors(
    State(_state): State<AppState>,
) -> Result<Json<Vec<Sensor>>, AppError> {
    // モックデータ
    let sensors = vec![
        Sensor {
            id: 1,
            name: "温度センサー #1".to_string(),
            sensor_type: "temperature".to_string(),
            location: "1F ロビー".to_string(),
        },
        Sensor {
            id: 2,
            name: "湿度センサー #1".to_string(),
            sensor_type: "humidity".to_string(),
            location: "1F ロビー".to_string(),
        },
        Sensor {
            id: 3,
            name: "電力メーター".to_string(),
            sensor_type: "power".to_string(),
            location: "サーバールーム".to_string(),
        },
        Sensor {
            id: 4,
            name: "心拍センサー".to_string(),
            sensor_type: "heartRate".to_string(),
            location: "ウェアラブル".to_string(),
        },
    ];

    Ok(Json(sensors))
}

// ============================================================================
// 店舗関連（PostGIS）
// ============================================================================

/// 店舗
#[derive(Debug, Serialize)]
pub struct Store {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lon: f64,
    pub distance: Option<f64>,
}

/// 近隣検索パラメータ
#[derive(Debug, Deserialize)]
pub struct NearbyQuery {
    pub lat: f64,
    pub lon: f64,
    pub radius: Option<f64>,
    pub limit: Option<i32>,
}

/// 近くの店舗検索（PostGIS ST_DWithin）
pub async fn get_nearby_stores(
    State(state): State<AppState>,
    Query(params): Query<NearbyQuery>,
) -> Result<Json<Vec<Store>>, AppError> {
    let radius = params.radius.unwrap_or(5000.0);
    let limit = params.limit.unwrap_or(10).min(50);

    let client = state.db.get().await?;

    // PostGIS のスキーマが存在する場合のクエリ
    let check_table = client
        .query_opt(
            "SELECT 1 FROM information_schema.tables
             WHERE table_schema = 'florida' AND table_name = 'points_of_interest'",
            &[],
        )
        .await?;

    if check_table.is_some() {
        let rows = client
            .query(
                r#"
                SELECT
                    id,
                    name,
                    COALESCE(amenity, 'store') AS address,
                    ST_Y(location::geometry) AS lat,
                    ST_X(location::geometry) AS lon,
                    ST_Distance(
                        location::geography,
                        ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography
                    ) AS distance
                FROM florida.points_of_interest
                WHERE ST_DWithin(
                    location::geography,
                    ST_SetSRID(ST_MakePoint($2, $1), 4326)::geography,
                    $3
                )
                ORDER BY distance
                LIMIT $4
                "#,
                &[&params.lat, &params.lon, &radius, &(limit as i64)],
            )
            .await?;

        let stores: Vec<Store> = rows
            .iter()
            .map(|row| Store {
                id: row.get("id"),
                name: row.get("name"),
                address: row.get("address"),
                lat: row.get("lat"),
                lon: row.get("lon"),
                distance: row.get("distance"),
            })
            .collect();

        Ok(Json(stores))
    } else {
        // モックデータを返す
        Ok(Json(generate_mock_stores(params.lat, params.lon, radius)))
    }
}

fn generate_mock_stores(user_lat: f64, user_lon: f64, radius: f64) -> Vec<Store> {
    let mock_stores = vec![
        ("Coffee Shop 渋谷店", "東京都渋谷区道玄坂1-1-1", 35.6595, 139.7004),
        ("Coffee Shop 新宿店", "東京都新宿区新宿3-1-1", 35.6896, 139.7006),
        ("Coffee Shop 池袋店", "東京都豊島区南池袋1-1-1", 35.7295, 139.7109),
        ("Coffee Shop 原宿店", "東京都渋谷区神宮前1-1-1", 35.6702, 139.7027),
        ("Coffee Shop 表参道店", "東京都港区北青山3-1-1", 35.6653, 139.7121),
    ];

    mock_stores
        .into_iter()
        .enumerate()
        .map(|(i, (name, address, lat, lon))| {
            // 簡易的な距離計算
            let dlat = (lat - user_lat).to_radians();
            let dlon = (lon - user_lon).to_radians();
            let a = (dlat / 2.0).sin().powi(2)
                + user_lat.to_radians().cos() * lat.to_radians().cos() * (dlon / 2.0).sin().powi(2);
            let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
            let distance = 6371000.0 * c; // 地球の半径（メートル）

            Store {
                id: (i + 1) as i32,
                name: name.to_string(),
                address: address.to_string(),
                lat,
                lon,
                distance: Some(distance),
            }
        })
        .filter(|s| s.distance.unwrap_or(f64::MAX) <= radius)
        .collect()
}

/// 全店舗一覧取得
pub async fn get_all_stores(
    State(state): State<AppState>,
) -> Result<Json<Vec<Store>>, AppError> {
    let client = state.db.get().await?;

    // stores.locations テーブルが存在するか確認
    let check_table = client
        .query_opt(
            "SELECT 1 FROM information_schema.tables
             WHERE table_schema = 'stores' AND table_name = 'locations'",
            &[],
        )
        .await?;

    if check_table.is_some() {
        let rows = client
            .query(
                r#"
                SELECT id, name, address, lat, lon
                FROM stores.locations
                ORDER BY prefecture, city, name
                LIMIT 100
                "#,
                &[],
            )
            .await?;

        let stores: Vec<Store> = rows
            .iter()
            .map(|row| Store {
                id: row.get("id"),
                name: row.get("name"),
                address: row.get("address"),
                lat: row.get("lat"),
                lon: row.get("lon"),
                distance: None,
            })
            .collect();

        Ok(Json(stores))
    } else {
        // モックデータを返す
        Ok(Json(generate_mock_stores(35.6595, 139.7004, 100000.0)))
    }
}

// ============================================================================
// ヘルスチェック
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

/// ヘルスチェック
pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    let db_status = match state.db.get().await {
        Ok(client) => {
            match client.query_one("SELECT 1", &[]).await {
                Ok(_) => "healthy".to_string(),
                Err(_) => "unhealthy".to_string(),
            }
        }
        Err(_) => "unhealthy".to_string(),
    };

    Ok(Json(HealthResponse {
        status: "ok".to_string(),
        database: db_status,
    }))
}

// ============================================================================
// ユーザー関連
// ============================================================================

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: String,
    pub phone: Option<String>,
    pub membership_tier: String,
    pub points: i32,
    pub preferences: Option<JsonValue>,
}

/// ユーザー取得
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<User>, AppError> {
    let client = state.db.get().await?;

    let row = client
        .query_opt(
            "SELECT id, email, name, phone, membership_tier, points, preferences FROM customers.users WHERE id = $1",
            &[&id],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;

    Ok(Json(User {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        phone: row.get("phone"),
        membership_tier: row.get("membership_tier"),
        points: row.get("points"),
        preferences: row.get("preferences"),
    }))
}

/// 全ユーザー取得
pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, AppError> {
    let client = state.db.get().await?;

    let rows = client
        .query(
            "SELECT id, email, name, phone, membership_tier, points, preferences FROM customers.users ORDER BY id",
            &[],
        )
        .await?;

    let users: Vec<User> = rows
        .iter()
        .map(|row| User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            phone: row.get("phone"),
            membership_tier: row.get("membership_tier"),
            points: row.get("points"),
            preferences: row.get("preferences"),
        })
        .collect();

    Ok(Json(users))
}

// ============================================================================
// 注文関連
// ============================================================================

#[derive(Debug, Serialize)]
pub struct Order {
    pub id: i32,
    pub order_number: String,
    pub user_id: Option<i32>,
    pub store_id: i32,
    pub status: String,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub points_earned: i32,
    pub payment_method: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct OrderItem {
    pub id: i32,
    pub product_id: i32,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
    pub customizations: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
pub struct OrderDetail {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Deserialize)]
pub struct OrdersQuery {
    pub user_id: Option<i32>,
    pub status: Option<String>,
    pub limit: Option<i32>,
}

/// 注文一覧取得
pub async fn get_orders(
    State(state): State<AppState>,
    Query(params): Query<OrdersQuery>,
) -> Result<Json<Vec<Order>>, AppError> {
    let client = state.db.get().await?;
    let limit = params.limit.unwrap_or(50).min(100);

    let rows = client
        .query(
            r#"
            SELECT id, order_number, user_id, store_id, status,
                   subtotal, tax, total, points_earned, payment_method, created_at
            FROM orders.orders
            WHERE ($1::INT IS NULL OR user_id = $1)
              AND ($2::TEXT IS NULL OR status = $2)
            ORDER BY created_at DESC
            LIMIT $3
            "#,
            &[&params.user_id, &params.status, &(limit as i64)],
        )
        .await?;

    let orders: Vec<Order> = rows
        .iter()
        .map(|row| Order {
            id: row.get("id"),
            order_number: row.get("order_number"),
            user_id: row.get("user_id"),
            store_id: row.get("store_id"),
            status: row.get("status"),
            subtotal: row.get("subtotal"),
            tax: row.get("tax"),
            total: row.get("total"),
            points_earned: row.get("points_earned"),
            payment_method: row.get("payment_method"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(orders))
}

/// 注文詳細取得
pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<OrderDetail>, AppError> {
    let client = state.db.get().await?;

    let order_row = client
        .query_opt(
            r#"
            SELECT id, order_number, user_id, store_id, status,
                   subtotal, tax, total, points_earned, payment_method, created_at
            FROM orders.orders WHERE id = $1
            "#,
            &[&id],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Order {} not found", id)))?;

    let order = Order {
        id: order_row.get("id"),
        order_number: order_row.get("order_number"),
        user_id: order_row.get("user_id"),
        store_id: order_row.get("store_id"),
        status: order_row.get("status"),
        subtotal: order_row.get("subtotal"),
        tax: order_row.get("tax"),
        total: order_row.get("total"),
        points_earned: order_row.get("points_earned"),
        payment_method: order_row.get("payment_method"),
        created_at: order_row.get("created_at"),
    };

    let item_rows = client
        .query(
            r#"
            SELECT id, product_id, product_name, quantity, unit_price, subtotal, customizations
            FROM orders.order_items WHERE order_id = $1 ORDER BY id
            "#,
            &[&id],
        )
        .await?;

    let items: Vec<OrderItem> = item_rows
        .iter()
        .map(|row| OrderItem {
            id: row.get("id"),
            product_id: row.get("product_id"),
            product_name: row.get("product_name"),
            quantity: row.get("quantity"),
            unit_price: row.get("unit_price"),
            subtotal: row.get("subtotal"),
            customizations: row.get("customizations"),
        })
        .collect();

    Ok(Json(OrderDetail { order, items }))
}

// ============================================================================
// お気に入り関連
// ============================================================================

#[derive(Debug, Serialize)]
pub struct Favorite {
    pub id: i32,
    pub user_id: i32,
    pub product_id: i32,
    pub product_name: Option<String>,
    pub product_category: Option<String>,
    pub product_price: Option<Decimal>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FavoritesQuery {
    pub user_id: i32,
}

/// お気に入り一覧取得
pub async fn get_favorites(
    State(state): State<AppState>,
    Query(params): Query<FavoritesQuery>,
) -> Result<Json<Vec<Favorite>>, AppError> {
    let client = state.db.get().await?;

    let rows = client
        .query(
            r#"
            SELECT f.id, f.user_id, f.product_id, f.created_at,
                   p.name as product_name, p.category as product_category, p.price as product_price
            FROM customers.favorites f
            LEFT JOIN products.catalog p ON f.product_id = p.id
            WHERE f.user_id = $1
            ORDER BY f.created_at DESC
            "#,
            &[&params.user_id],
        )
        .await?;

    let favorites: Vec<Favorite> = rows
        .iter()
        .map(|row| Favorite {
            id: row.get("id"),
            user_id: row.get("user_id"),
            product_id: row.get("product_id"),
            product_name: row.get("product_name"),
            product_category: row.get("product_category"),
            product_price: row.get("product_price"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(favorites))
}

#[derive(Debug, Deserialize)]
pub struct AddFavoriteRequest {
    pub user_id: i32,
    pub product_id: i32,
}

/// お気に入り追加
pub async fn add_favorite(
    State(state): State<AppState>,
    Json(req): Json<AddFavoriteRequest>,
) -> Result<Json<Favorite>, AppError> {
    let client = state.db.get().await?;

    let row = client
        .query_one(
            r#"
            INSERT INTO customers.favorites (user_id, product_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, product_id) DO UPDATE SET user_id = $1
            RETURNING id, user_id, product_id, created_at
            "#,
            &[&req.user_id, &req.product_id],
        )
        .await?;

    Ok(Json(Favorite {
        id: row.get("id"),
        user_id: row.get("user_id"),
        product_id: row.get("product_id"),
        product_name: None,
        product_category: None,
        product_price: None,
        created_at: row.get("created_at"),
    }))
}

/// お気に入り削除
pub async fn delete_favorite(
    State(state): State<AppState>,
    Path((user_id, product_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let client = state.db.get().await?;

    client
        .execute(
            "DELETE FROM customers.favorites WHERE user_id = $1 AND product_id = $2",
            &[&user_id, &product_id],
        )
        .await?;

    Ok(Json(serde_json::json!({"success": true})))
}

// ============================================================================
// レビュー関連
// ============================================================================

#[derive(Debug, Serialize)]
pub struct Review {
    pub id: i32,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub product_id: Option<i32>,
    pub store_id: Option<i32>,
    pub rating: i32,
    pub title: Option<String>,
    pub content: Option<String>,
    pub helpful_count: i32,
    pub verified_purchase: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewsQuery {
    pub product_id: Option<i32>,
    pub store_id: Option<i32>,
    pub limit: Option<i32>,
}

/// レビュー一覧取得
pub async fn get_reviews(
    State(state): State<AppState>,
    Query(params): Query<ReviewsQuery>,
) -> Result<Json<Vec<Review>>, AppError> {
    let client = state.db.get().await?;
    let limit = params.limit.unwrap_or(20).min(50);

    let rows = client
        .query(
            r#"
            SELECT r.id, r.user_id, u.name as user_name, r.product_id, r.store_id,
                   r.rating, r.title, r.content, r.helpful_count, r.verified_purchase, r.created_at
            FROM customers.reviews r
            LEFT JOIN customers.users u ON r.user_id = u.id
            WHERE ($1::INT IS NULL OR r.product_id = $1)
              AND ($2::INT IS NULL OR r.store_id = $2)
            ORDER BY r.created_at DESC
            LIMIT $3
            "#,
            &[&params.product_id, &params.store_id, &(limit as i64)],
        )
        .await?;

    let reviews: Vec<Review> = rows
        .iter()
        .map(|row| Review {
            id: row.get("id"),
            user_id: row.get("user_id"),
            user_name: row.get("user_name"),
            product_id: row.get("product_id"),
            store_id: row.get("store_id"),
            rating: row.get("rating"),
            title: row.get("title"),
            content: row.get("content"),
            helpful_count: row.get("helpful_count"),
            verified_purchase: row.get("verified_purchase"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(Json(reviews))
}

// ============================================================================
// 分析・統計関連
// ============================================================================

#[derive(Debug, Serialize)]
pub struct SalesStats {
    pub total_orders: i64,
    pub total_revenue: Decimal,
    pub avg_order_value: Decimal,
    pub total_items_sold: i64,
}

#[derive(Debug, Serialize)]
pub struct DailySales {
    pub date: String,
    pub orders: i64,
    pub revenue: Decimal,
}

#[derive(Debug, Serialize)]
pub struct TopProduct {
    pub product_id: i32,
    pub product_name: String,
    pub quantity_sold: i64,
    pub revenue: Decimal,
}

#[derive(Debug, Serialize)]
pub struct AnalyticsDashboard {
    pub stats: SalesStats,
    pub daily_sales: Vec<DailySales>,
    pub top_products: Vec<TopProduct>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<i32>,
}

/// 分析ダッシュボード
pub async fn get_analytics(
    State(state): State<AppState>,
    Query(params): Query<AnalyticsQuery>,
) -> Result<Json<AnalyticsDashboard>, AppError> {
    let client = state.db.get().await?;
    let days = params.days.unwrap_or(30).min(90);

    // 全体統計
    let stats_row = client
        .query_one(
            r#"
            SELECT
                COUNT(*)::BIGINT as total_orders,
                COALESCE(SUM(total), 0) as total_revenue,
                COALESCE(AVG(total), 0) as avg_order_value,
                COALESCE(SUM((SELECT SUM(quantity) FROM orders.order_items WHERE order_id = o.id)), 0)::BIGINT as total_items_sold
            FROM orders.orders o
            WHERE status = 'completed'
              AND created_at >= NOW() - ($1 || ' days')::INTERVAL
            "#,
            &[&days.to_string()],
        )
        .await?;

    let stats = SalesStats {
        total_orders: stats_row.get("total_orders"),
        total_revenue: stats_row.get("total_revenue"),
        avg_order_value: stats_row.get("avg_order_value"),
        total_items_sold: stats_row.get("total_items_sold"),
    };

    // 日別売上
    let daily_rows = client
        .query(
            r#"
            SELECT
                DATE(created_at) as date,
                COUNT(*)::BIGINT as orders,
                SUM(total) as revenue
            FROM orders.orders
            WHERE status = 'completed'
              AND created_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY DATE(created_at)
            ORDER BY date DESC
            "#,
            &[&days.to_string()],
        )
        .await?;

    let daily_sales: Vec<DailySales> = daily_rows
        .iter()
        .map(|row| {
            let date: chrono::NaiveDate = row.get("date");
            DailySales {
                date: date.to_string(),
                orders: row.get("orders"),
                revenue: row.get("revenue"),
            }
        })
        .collect();

    // 人気商品
    let top_rows = client
        .query(
            r#"
            SELECT
                oi.product_id,
                oi.product_name,
                SUM(oi.quantity)::BIGINT as quantity_sold,
                SUM(oi.subtotal) as revenue
            FROM orders.order_items oi
            JOIN orders.orders o ON oi.order_id = o.id
            WHERE o.status = 'completed'
              AND o.created_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY oi.product_id, oi.product_name
            ORDER BY quantity_sold DESC
            LIMIT 10
            "#,
            &[&days.to_string()],
        )
        .await?;

    let top_products: Vec<TopProduct> = top_rows
        .iter()
        .map(|row| TopProduct {
            product_id: row.get("product_id"),
            product_name: row.get("product_name"),
            quantity_sold: row.get("quantity_sold"),
            revenue: row.get("revenue"),
        })
        .collect();

    Ok(Json(AnalyticsDashboard {
        stats,
        daily_sales,
        top_products,
    }))
}

// ============================================================================
// 店舗詳細関連
// ============================================================================

#[derive(Debug, Serialize)]
pub struct StoreDetail {
    pub id: i32,
    pub name: String,
    pub address: String,
    pub city: String,
    pub prefecture: String,
    pub phone: Option<String>,
    pub lat: f64,
    pub lon: f64,
    pub store_type: String,
    pub opening_hours: Option<JsonValue>,
    pub facilities: Option<Vec<String>>,
    pub avg_rating: Option<f64>,
    pub review_count: i64,
}

/// 店舗詳細取得
pub async fn get_store(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<StoreDetail>, AppError> {
    let client = state.db.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT s.id, s.name, s.address, s.city, s.prefecture, s.phone,
                   s.lat, s.lon, s.store_type, s.opening_hours, s.facilities,
                   COALESCE(AVG(r.rating)::FLOAT8, 0) as avg_rating,
                   COUNT(r.id)::BIGINT as review_count
            FROM stores.locations s
            LEFT JOIN customers.reviews r ON s.id = r.store_id
            WHERE s.id = $1
            GROUP BY s.id
            "#,
            &[&id],
        )
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Store {} not found", id)))?;

    Ok(Json(StoreDetail {
        id: row.get("id"),
        name: row.get("name"),
        address: row.get("address"),
        city: row.get("city"),
        prefecture: row.get("prefecture"),
        phone: row.get("phone"),
        lat: row.get("lat"),
        lon: row.get("lon"),
        store_type: row.get("store_type"),
        opening_hours: row.get("opening_hours"),
        facilities: row.get("facilities"),
        avg_rating: {
            let val: f64 = row.get("avg_rating");
            if val > 0.0 { Some(val) } else { None }
        },
        review_count: row.get("review_count"),
    }))
}
