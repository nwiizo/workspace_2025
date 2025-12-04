//! Data models used across the API examples

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User claims extracted from JWT token
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Permissions/roles
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Expiration time
    pub exp: usize,
    /// Issued at
    #[serde(default)]
    pub iat: usize,
    /// Audience
    #[serde(default)]
    pub aud: Option<String>,
    /// Issuer
    #[serde(default)]
    pub iss: Option<String>,
}

/// Order model for BOLA demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: i64,
    pub user: String,
    pub product: String,
    pub quantity: i32,
}

/// Create order request
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub product: String,
    pub quantity: i32,
}

/// Payment model for mass assignment demonstration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}

/// Safe payment creation request (whitelisted fields only)
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: f64,
    pub currency: String,
}

/// Unsafe payment creation - accepts any fields (vulnerable to mass assignment)
#[derive(Debug, Deserialize)]
pub struct UnsafePaymentRequest {
    pub amount: f64,
    pub currency: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

/// User model for authentication examples
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: String,
}

/// User response (without sensitive data)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub email: String,
    pub role: String,
}

/// User response with excessive data exposure (vulnerable)
#[derive(Debug, Serialize)]
pub struct UserResponseVulnerable {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub internal_notes: String,
    pub created_at: String,
}

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
}

/// URL fetch request for SSRF examples
#[derive(Debug, Deserialize)]
pub struct FetchUrlRequest {
    pub url: String,
}

impl Payment {
    pub fn new(amount: f64, currency: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            amount,
            currency,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
