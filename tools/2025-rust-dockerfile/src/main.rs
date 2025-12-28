use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Health check response structure
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

/// User information structure
#[derive(Debug, Deserialize, Serialize)]
struct User {
    name: String,
    email: String,
}

/// Creates and returns the application router with all routes configured
///
/// # Returns
///
/// Returns a configured `Router` with health check and user endpoints
///
/// # Examples
///
/// ```no_run
/// let app = app();
/// // Serve with axum
/// ```
fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/users", post(create_user))
}

/// Root endpoint handler
///
/// # Returns
///
/// Returns a welcome message as JSON
async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "message": "Welcome to Rust Docker Demo 2025!",
        "version": "0.1.0"
    }))
}

/// Health check endpoint handler
///
/// Provides basic health status of the application
///
/// # Returns
///
/// Returns health status as JSON
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Service is healthy".to_string(),
    })
}

/// Create user endpoint handler
///
/// Accepts user data and returns the created user information
///
/// # Arguments
///
/// * `user` - User data from request body
///
/// # Returns
///
/// Returns the created user data with 201 status code
async fn create_user(Json(user): Json<User>) -> (StatusCode, Json<User>) {
    (StatusCode::CREATED, Json(user))
}

/// Application entry point
///
/// Starts the Axum web server on port 8000
///
/// # Errors
///
/// Returns error if server fails to bind to the specified address
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get port from environment variable or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("🚀 Server starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app()).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_root_endpoint() {
        let app = app();

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
