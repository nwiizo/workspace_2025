//! ルーター定義
//!
//! Axum の Router を構築します。

use axum::{
    routing::{get, post, delete},
    Router,
};
use axum_extra::extract::cookie::Key;
use tower_http::cors::CorsLayer;
use axum::http::{header, Method};

use super::{auth, handlers, AppState};

/// ルーターを作成
pub fn create_router(state: AppState) -> Router {
    // CORS 設定（開発用：credentials を許可する場合は明示的な設定が必要）
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::COOKIE])
        .allow_credentials(true);

    // 認証ルート
    let auth_routes = Router::new()
        .route("/google/login", get(auth::google_login))
        .route("/google/callback", get(auth::google_callback))
        .route("/logout", post(auth::logout))
        .route("/status", get(auth::get_auth_status))
        .route("/me", get(auth::get_current_user))
        // 開発用モック認証（DEV_AUTH_ENABLED=true で有効）
        .route("/dev/login", get(auth::dev_login));

    // API ルート
    let api_routes = Router::new()
        // ヘルスチェック
        .route("/health", get(handlers::health_check))
        // 認証 API
        .nest("/auth", auth_routes)
        // 商品 API
        .route("/products/search", get(handlers::search_products))
        .route("/products/categories", get(handlers::get_categories))
        .route("/products/{id}", get(handlers::get_product))
        .route("/products/{id}/similar", get(handlers::get_similar_products))
        // メトリクス API（TimescaleDB）
        .route("/metrics", get(handlers::get_metrics))
        .route("/sensors", get(handlers::get_sensors))
        // 店舗 API（PostGIS）
        .route("/stores", get(handlers::get_all_stores))
        .route("/stores/nearby", get(handlers::get_nearby_stores))
        .route("/stores/{id}", get(handlers::get_store))
        // ユーザー API
        .route("/users", get(handlers::get_users))
        .route("/users/{id}", get(handlers::get_user))
        // 注文 API
        .route("/orders", get(handlers::get_orders))
        .route("/orders/{id}", get(handlers::get_order))
        // お気に入り API
        .route("/favorites", get(handlers::get_favorites).post(handlers::add_favorite))
        .route("/favorites/{user_id}/{product_id}", delete(handlers::delete_favorite))
        // レビュー API
        .route("/reviews", get(handlers::get_reviews))
        // 分析 API
        .route("/analytics", get(handlers::get_analytics));

    // Cookie 署名用の鍵を取得
    let cookie_key = state.cookie_key.clone();

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(axum::Extension(cookie_key))
        .with_state(state)
}

/// FromRef 実装で PrivateCookieJar が Key を取得できるようにする
impl axum::extract::FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}
