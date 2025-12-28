use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// User data structure for API requests/responses
#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

/// Application state for storing users
struct AppState {
    users: Mutex<Vec<User>>,
}

/// Health check endpoint
///
/// # Returns
///
/// Returns "OK" message indicating the service is healthy
///
/// # Example
///
/// ```
/// GET /health
/// Response: {"status": "OK"}
/// ```
#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "OK",
        "service": "sample-api"
    }))
}

/// Get all users
///
/// # Returns
///
/// Returns a JSON array of all users
///
/// # Example
///
/// ```
/// GET /users
/// Response: [{"id": 1, "name": "Alice", "email": "alice@example.com"}]
/// ```
#[get("/users")]
async fn get_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    HttpResponse::Ok().json(&*users)
}

/// Create a new user
///
/// # Arguments
///
/// * `user` - User data from request body
///
/// # Returns
///
/// Returns the created user with HTTP 201 status
///
/// # Example
///
/// ```
/// POST /users
/// Body: {"id": 1, "name": "Alice", "email": "alice@example.com"}
/// Response: {"id": 1, "name": "Alice", "email": "alice@example.com"}
/// ```
#[post("/users")]
async fn create_user(user: web::Json<User>, data: web::Data<AppState>) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    users.push(user.clone());
    log::info!("Created user: {:?}", user);
    HttpResponse::Created().json(user.0)
}

/// Main entry point for the API server
///
/// Initializes the server with routes and starts listening on port 8080
///
/// # Returns
///
/// Returns `std::io::Result` indicating success or failure
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Starting sample API server...");

    // Initialize application state with sample data
    let app_state = web::Data::new(AppState {
        users: Mutex::new(vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ]),
    });

    log::info!("Server will listen on http://0.0.0.0:8080");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(health_check)
            .service(get_users)
            .service(create_user)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
// Test comment to trigger rebuild
