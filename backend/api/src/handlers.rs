//! Request handlers for the Intents API
//! 
//! This module contains reusable handler functions that can be shared
//! across different route modules.

use axum::{
    extract::State,
    response::Json,
};
use ethers::types::{Address, H256};

use crate::{
    models::AppState,
    error::Result,
    cache::CacheService,
    websocket::broadcast_intent_update,
};

/// Update intent status and broadcast to subscribers
pub async fn update_intent_status_and_broadcast(
    state: &AppState,
    intent_id: H256,
    status: &str,
    progress_step: &str,
    progress_percentage: f64,
    details: Option<serde_json::Value>,
) -> Result<()> {
    // Update cache
    let mut cache = CacheService::new(state.redis.clone());
    
    let status_response = crate::models::IntentStatusResponse {
        intent_id,
        status: status.to_string(),
        progress: crate::models::IntentProgress {
            current_step: progress_step.to_string(),
            steps_completed: (progress_percentage / 20.0) as u32, // Assuming 5 steps total
            total_steps: 5,
            percentage: progress_percentage,
        },
        estimated_completion: if status == "completed" || status == "failed" {
            None
        } else {
            Some(chrono::Utc::now() + chrono::Duration::minutes(5))
        },
        error_message: if status == "failed" {
            Some("Execution failed".to_string())
        } else {
            None
        },
    };
    
    cache.cache_intent_status(intent_id, &status_response).await?;
    
    // Broadcast update
    let update_msg = crate::models::IntentUpdateMessage {
        intent_id,
        status: status.to_string(),
        progress: status_response.progress,
        details,
    };
    
    broadcast_intent_update(intent_id, update_msg).await;
    
    Ok(())
}

/// Common error response handler
pub fn create_error_response(error: &str, code: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "error": {
            "code": code,
            "message": error,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    }))
}

/// Success response helper
pub fn create_success_response(message: &str, data: Option<serde_json::Value>) -> Json<serde_json::Value> {
    let mut response = serde_json::json!({
        "success": true,
        "message": message,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    if let Some(data) = data {
        response["data"] = data;
    }
    
    Json(response)
}

/// Validate Ethereum address format
pub fn validate_eth_address(address_str: &str) -> Result<Address> {
    address_str.parse::<Address>()
        .map_err(|_| crate::error::validation_error("Invalid Ethereum address format"))
}

/// Validate intent ID format
pub fn validate_intent_id(intent_id_str: &str) -> Result<H256> {
    intent_id_str.parse::<H256>()
        .map_err(|_| crate::error::validation_error("Invalid intent ID format"))
}

/// Get user-friendly error message based on error type
pub fn get_user_friendly_error(error: &crate::error::ApiError) -> &'static str {
    match error {
        crate::error::ApiError::Validation(_) => "Invalid input data",
        crate::error::ApiError::Authentication(_) => "Authentication required",
        crate::error::ApiError::Authorization(_) => "Access denied",
        crate::error::ApiError::NotFound(_) => "Resource not found",
        crate::error::ApiError::RateLimit => "Too many requests",
        crate::error::ApiError::Database(_) => "Database error",
        crate::error::ApiError::Redis(_) => "Cache error",
        crate::error::ApiError::IntentEngine(_) => "Intent processing error",
        crate::error::ApiError::Blockchain(_) => "Blockchain communication error",
        _ => "Internal server error",
    }
}