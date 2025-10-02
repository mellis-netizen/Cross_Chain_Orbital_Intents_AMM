use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use ethers::types::Address;
use std::str::FromStr;

use crate::{
    models::AppState,
    auth::*,
    cache::CacheService,
    error::{Result, validation_error},
};

// Authentication routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/challenge", post(get_auth_challenge))
        .route("/login", post(authenticate))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh_token))
        .route("/profile", get(get_profile))
}

// Get authentication challenge
async fn get_auth_challenge(
    Json(request): Json<AuthChallengeRequest>,
) -> Result<Json<AuthChallengeResponse>> {
    let challenge = generate_auth_challenge(request.address);
    Ok(Json(challenge))
}

// Authenticate user with signed message
async fn authenticate(
    State(state): State<AppState>,
    Json(request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate the authentication request
    validate_auth_challenge(&request, 300)?; // 5 minute expiry
    
    // Determine user role
    let role = UserRole::from_address(request.address);
    
    // Generate JWT token
    let jwt_manager = JwtManager::new(&state.config.jwt_secret);
    let token = jwt_manager.generate_token(
        request.address,
        role.as_str(),
        chrono::Duration::hours(24), // 24 hour expiry
    )?;
    
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp();
    
    // Create session
    let mut cache = CacheService::new(state.redis.clone());
    let session_expires = chrono::Utc::now() + chrono::Duration::hours(24);
    SessionManager::create_session(&mut cache, request.address, &token, session_expires).await?;
    
    tracing::info!("User authenticated: {:#x} with role: {}", request.address, role.as_str());
    
    Ok(Json(AuthResponse {
        token,
        expires_at,
        user_address: request.address,
        role: role.as_str().to_string(),
    }))
}

// Logout user
async fn logout(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<serde_json::Value>> {
    let user_address = extract_user_address(&claims)?;
    
    // Revoke session
    let mut cache = CacheService::new(state.redis.clone());
    SessionManager::revoke_session(&mut cache, user_address).await?;
    
    tracing::info!("User logged out: {:#x}", user_address);
    
    Ok(Json(serde_json::json!({
        "message": "Successfully logged out"
    })))
}

// Refresh JWT token
async fn refresh_token(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<AuthResponse>> {
    let user_address = extract_user_address(&claims)?;
    
    // Validate current session
    let mut cache = CacheService::new(state.redis.clone());
    // Note: In a production system, you'd want to validate the current token
    
    // Generate new token
    let jwt_manager = JwtManager::new(&state.config.jwt_secret);
    let new_token = jwt_manager.generate_token(
        user_address,
        &claims.role,
        chrono::Duration::hours(24),
    )?;
    
    let expires_at = (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp();
    
    // Update session
    let session_expires = chrono::Utc::now() + chrono::Duration::hours(24);
    SessionManager::create_session(&mut cache, user_address, &new_token, session_expires).await?;
    
    Ok(Json(AuthResponse {
        token: new_token,
        expires_at,
        user_address,
        role: claims.role,
    }))
}

// Get user profile
async fn get_profile(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<serde_json::Value>> {
    let user_address = extract_user_address(&claims)?;
    
    // Get user statistics
    let user_stats = get_user_statistics(&state, user_address).await?;
    
    Ok(Json(serde_json::json!({
        "address": format!("{:#x}", user_address),
        "role": claims.role,
        "statistics": user_stats
    })))
}

// Helper function to get user statistics
async fn get_user_statistics(
    state: &AppState,
    user_address: Address,
) -> Result<serde_json::Value> {
    // Get basic intent statistics for the user
    let total_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE user_address = $1"
    )
    .bind(format!("{:#x}", user_address))
    .fetch_one(&state.db)
    .await?;
    
    let successful_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE user_address = $1 AND status = 'completed'"
    )
    .bind(format!("{:#x}", user_address))
    .fetch_one(&state.db)
    .await?;
    
    let pending_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE user_address = $1 AND status = 'pending'"
    )
    .bind(format!("{:#x}", user_address))
    .fetch_one(&state.db)
    .await?;
    
    // Calculate total volume
    let total_volume: Option<(String,)> = sqlx::query_as(
        "SELECT SUM(source_amount) FROM intents WHERE user_address = $1 AND status = 'completed'"
    )
    .bind(format!("{:#x}", user_address))
    .fetch_optional(&state.db)
    .await?;
    
    let volume_str = total_volume
        .and_then(|(sum,)| sum)
        .unwrap_or_else(|| "0".to_string());
    
    Ok(serde_json::json!({
        "total_intents": total_intents.0,
        "successful_intents": successful_intents.0,
        "pending_intents": pending_intents.0,
        "total_volume": volume_str,
        "success_rate": if total_intents.0 > 0 {
            successful_intents.0 as f64 / total_intents.0 as f64
        } else {
            0.0
        }
    }))
}