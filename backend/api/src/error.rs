use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApiError>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Intent engine error: {0}")]
    IntentEngine(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Blockchain error: {0}")]
    Blockchain(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ApiError::Database(ref e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database operation failed")
            }
            ApiError::Redis(ref msg) => {
                tracing::error!("Redis error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "CACHE_ERROR", "Cache operation failed")
            }
            ApiError::IntentEngine(ref msg) => {
                tracing::error!("Intent engine error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTENT_ENGINE_ERROR", msg.as_str())
            }
            ApiError::Validation(ref msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.as_str())
            }
            ApiError::Authentication(ref msg) => {
                (StatusCode::UNAUTHORIZED, "AUTHENTICATION_ERROR", msg.as_str())
            }
            ApiError::Authorization(ref msg) => {
                (StatusCode::FORBIDDEN, "AUTHORIZATION_ERROR", msg.as_str())
            }
            ApiError::RateLimit => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED", "Too many requests")
            }
            ApiError::NotFound(ref msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", msg.as_str())
            }
            ApiError::BadRequest(ref msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.as_str())
            }
            ApiError::Conflict(ref msg) => {
                (StatusCode::CONFLICT, "CONFLICT", msg.as_str())
            }
            ApiError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error")
            }
            ApiError::ServiceUnavailable(ref msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE", msg.as_str())
            }
            ApiError::Blockchain(ref msg) => {
                tracing::error!("Blockchain error: {}", msg);
                (StatusCode::BAD_GATEWAY, "BLOCKCHAIN_ERROR", msg.as_str())
            }
            ApiError::Serialization(ref e) => {
                tracing::error!("Serialization error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR", "Serialization failed")
            }
            ApiError::Jwt(ref e) => {
                tracing::debug!("JWT error: {}", e);
                (StatusCode::UNAUTHORIZED, "JWT_ERROR", "Invalid or expired token")
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (status, body).into_response()
    }
}

// Helper function for creating validation errors
pub fn validation_error(msg: impl Into<String>) -> ApiError {
    ApiError::Validation(msg.into())
}

// Helper function for creating not found errors
pub fn not_found(resource: impl Into<String>) -> ApiError {
    ApiError::NotFound(format!("{} not found", resource.into()))
}

// Helper function for creating internal errors
pub fn internal_error(msg: impl Into<String>) -> ApiError {
    ApiError::Internal(msg.into())
}