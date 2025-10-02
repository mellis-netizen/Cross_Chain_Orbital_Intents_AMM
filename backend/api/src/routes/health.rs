use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    models::{AppState, HealthResponse, HealthCheck, ChainHealth},
    cache::CacheService,
    error::Result,
};

// Health check routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
}

// Main health check endpoint
async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthResponse>> {
    let start_time = std::time::Instant::now();
    
    // Check database
    let db_health = check_database_health(&state).await;
    
    // Check Redis
    let redis_health = check_redis_health(&state).await;
    
    // Check intent engine
    let engine_health = check_intent_engine_health(&state).await;
    
    // Check chains
    let chains_health = check_chains_health(&state).await;
    
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let overall_status = if db_health.status == "healthy" && 
                           redis_health.status == "healthy" && 
                           engine_health.status == "healthy" {
        "healthy"
    } else {
        "unhealthy"
    };
    
    let response = HealthResponse {
        status: overall_status.to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        database: db_health,
        redis: redis_health,
        intent_engine: engine_health,
        chains: chains_health,
    };
    
    tracing::debug!(
        "Health check completed in {:?}ms, status: {}",
        start_time.elapsed().as_millis(),
        overall_status
    );
    
    Ok(Json(response))
}

// Readiness check (for Kubernetes)
async fn readiness_check(
    State(state): State<AppState>,
) -> Result<StatusCode> {
    // Check if all critical services are ready
    let db_health = check_database_health(&state).await;
    let redis_health = check_redis_health(&state).await;
    
    if db_health.status == "healthy" && redis_health.status == "healthy" {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// Liveness check (for Kubernetes)
async fn liveness_check() -> Result<StatusCode> {
    // Simple check that the service is running
    Ok(StatusCode::OK)
}

// Database health check
async fn check_database_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();
    
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => HealthCheck {
            status: "healthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error_message: None,
        },
        Err(e) => HealthCheck {
            status: "unhealthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error_message: Some(e.to_string()),
        },
    }
}

// Redis health check
async fn check_redis_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();
    let mut cache = CacheService::new(state.redis.clone());
    
    match cache.health_check().await {
        Ok(true) => HealthCheck {
            status: "healthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error_message: None,
        },
        Ok(false) => HealthCheck {
            status: "unhealthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error_message: Some("Redis health check failed".to_string()),
        },
        Err(e) => HealthCheck {
            status: "unhealthy".to_string(),
            latency_ms: Some(start.elapsed().as_millis() as u64),
            error_message: Some(e.to_string()),
        },
    }
}

// Intent engine health check
async fn check_intent_engine_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();
    
    // TODO: Implement actual health check in intent engine
    // For now, just check if it's accessible
    HealthCheck {
        status: "healthy".to_string(),
        latency_ms: Some(start.elapsed().as_millis() as u64),
        error_message: None,
    }
}

// Chain health checks
async fn check_chains_health(state: &AppState) -> Vec<ChainHealth> {
    let mut chain_healths = Vec::new();
    
    for chain_config in &state.config.chains {
        let start = std::time::Instant::now();
        
        // Try to get latest block number
        let (status, block_number, gas_price, error_message) = 
            check_single_chain_health(chain_config).await;
        
        chain_healths.push(ChainHealth {
            chain_id: chain_config.chain_id,
            chain_name: chain_config.name.clone(),
            status,
            block_number,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            gas_price,
        });
    }
    
    chain_healths
}

// Check individual chain health
async fn check_single_chain_health(
    chain_config: &crate::config::ChainConfig,
) -> (String, Option<u64>, Option<ethers::types::U256>, Option<String>) {
    use ethers::providers::{Provider, Http};
    use std::str::FromStr;
    
    match Provider::<Http>::try_from(&chain_config.rpc_url) {
        Ok(provider) => {
            // Check latest block
            match provider.get_block_number().await {
                Ok(block_number) => {
                    // Try to get gas price
                    let gas_price = provider.get_gas_price().await.ok();
                    
                    (
                        "healthy".to_string(),
                        Some(block_number.as_u64()),
                        gas_price,
                        None,
                    )
                }
                Err(e) => (
                    "unhealthy".to_string(),
                    None,
                    None,
                    Some(format!("Failed to get block number: {}", e)),
                ),
            }
        }
        Err(e) => (
            "unhealthy".to_string(),
            None,
            None,
            Some(format!("Failed to create provider: {}", e)),
        ),
    }
}