use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use metrics_exporter_prometheus::PrometheusHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    models::AppState,
    error::Result,
    database::IntentDb,
    cache::CacheService,
};

// Metrics routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/metrics", get(prometheus_metrics))
        .route("/metrics/dashboard", get(dashboard_metrics))
}

// Prometheus metrics endpoint
async fn prometheus_metrics(
    State(state): State<AppState>,
) -> Result<Response> {
    // Update metrics before serving
    update_business_metrics(&state).await?;
    
    // Serve Prometheus metrics
    let metrics = state.prometheus_handle.render();
    
    Ok((
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        metrics,
    ).into_response())
}

// Dashboard metrics (JSON format)
async fn dashboard_metrics(
    State(state): State<AppState>,
) -> Result<axum::Json<serde_json::Value>> {
    let mut cache = CacheService::new(state.redis.clone());
    
    // Try to get cached analytics
    if let Some(cached_analytics) = cache.get_analytics().await? {
        return Ok(axum::Json(serde_json::to_value(cached_analytics)?));
    }
    
    // Generate fresh metrics
    let analytics = generate_analytics_data(&state).await?;
    
    // Cache for future requests
    cache.cache_analytics(&analytics).await.ok();
    
    Ok(axum::Json(serde_json::to_value(analytics)?))
}

// Update business metrics for Prometheus
async fn update_business_metrics(state: &AppState) -> Result<()> {
    // Get basic counts from database
    let total_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents"
    )
    .fetch_one(&state.db)
    .await?;
    
    let successful_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE status = 'completed'"
    )
    .fetch_one(&state.db)
    .await?;
    
    let failed_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE status = 'failed'"
    )
    .fetch_one(&state.db)
    .await?;
    
    let pending_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE status = 'pending'"
    )
    .fetch_one(&state.db)
    .await?;
    
    let active_solvers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM solvers WHERE is_active = true AND is_slashed = false"
    )
    .fetch_one(&state.db)
    .await?;
    
    // Update Prometheus metrics
    metrics::gauge!("intents_total").set(total_intents.0 as f64);
    metrics::gauge!("intents_successful").set(successful_intents.0 as f64);
    metrics::gauge!("intents_failed").set(failed_intents.0 as f64);
    metrics::gauge!("intents_pending").set(pending_intents.0 as f64);
    metrics::gauge!("solvers_active").set(active_solvers.0 as f64);
    
    // Calculate success rate
    let success_rate = if total_intents.0 > 0 {
        successful_intents.0 as f64 / total_intents.0 as f64 * 100.0
    } else {
        0.0
    };
    metrics::gauge!("intent_success_rate_percent").set(success_rate);
    
    // Update WebSocket metrics
    let ws_metrics = crate::websocket::get_websocket_metrics().await;
    if let Some(active_connections) = ws_metrics.get("active_connections").and_then(|v| v.as_u64()) {
        metrics::gauge!("websocket_active_connections").set(active_connections as f64);
    }
    
    Ok(())
}

// Generate comprehensive analytics data
async fn generate_analytics_data(state: &AppState) -> Result<crate::models::AnalyticsResponse> {
    // Basic intent statistics
    let total_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents"
    )
    .fetch_one(&state.db)
    .await?;
    
    let successful_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE status = 'completed'"
    )
    .fetch_one(&state.db)
    .await?;
    
    let failed_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE status = 'failed'"
    )
    .fetch_one(&state.db)
    .await?;
    
    let pending_intents: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM intents WHERE status = 'pending'"
    )
    .fetch_one(&state.db)
    .await?;
    
    // Calculate total volume
    let total_volume_result: Option<(String,)> = sqlx::query_as(
        "SELECT SUM(source_amount) FROM intents WHERE status = 'completed'"
    )
    .fetch_optional(&state.db)
    .await?;
    
    let total_volume = total_volume_result
        .and_then(|(sum,)| sum.parse::<ethers::types::U256>().ok())
        .unwrap_or_default();
    
    // Calculate average execution time
    let avg_execution_time: Option<(f64,)> = sqlx::query_as(r#"
        SELECT AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) 
        FROM intents 
        WHERE status = 'completed'
    "#)
    .fetch_optional(&state.db)
    .await?;
    
    let average_execution_time = avg_execution_time
        .map(|(avg,)| avg)
        .unwrap_or(0.0);
    
    // Calculate success rate
    let success_rate = if total_intents.0 > 0 {
        successful_intents.0 as f64 / total_intents.0 as f64
    } else {
        0.0
    };
    
    // Get top chains by volume
    let top_chains = get_top_chains_stats(&state.db).await?;
    
    // Get top tokens by volume
    let top_tokens = get_top_tokens_stats(&state.db).await?;
    
    // Get solver performance
    let solver_performance = get_solver_performance_stats(&state.db).await?;
    
    Ok(crate::models::AnalyticsResponse {
        total_intents: total_intents.0 as u64,
        successful_intents: successful_intents.0 as u64,
        failed_intents: failed_intents.0 as u64,
        pending_intents: pending_intents.0 as u64,
        total_volume,
        average_execution_time,
        success_rate,
        top_chains,
        top_tokens,
        solver_performance,
    })
}

// Get top chains by volume and activity
async fn get_top_chains_stats(
    db: &sqlx::PgPool,
) -> Result<Vec<crate::models::ChainStats>> {
    let records = sqlx::query!(r#"
        SELECT 
            source_chain_id as chain_id,
            COUNT(*) as intent_count,
            COALESCE(SUM(CASE WHEN source_amount != '' THEN source_amount::numeric ELSE 0 END), 0) as volume,
            COALESCE(AVG(CASE WHEN status = 'completed' THEN 1.0 ELSE 0.0 END), 0) as success_rate
        FROM intents 
        GROUP BY source_chain_id 
        ORDER BY volume DESC 
        LIMIT 10
    "#)
    .fetch_all(db)
    .await?;
    
    let mut chain_stats = Vec::new();
    
    for record in records {
        let chain_name = get_chain_name(record.chain_id as u64);
        
        chain_stats.push(crate::models::ChainStats {
            chain_id: record.chain_id as u64,
            chain_name,
            intent_count: record.intent_count.unwrap_or(0) as u64,
            volume: ethers::types::U256::from_dec_str(&record.volume.unwrap_or_default().to_string())
                .unwrap_or_default(),
            success_rate: record.success_rate.unwrap_or(0.0),
        });
    }
    
    Ok(chain_stats)
}

// Get top tokens by volume
async fn get_top_tokens_stats(
    db: &sqlx::PgPool,
) -> Result<Vec<crate::models::TokenStats>> {
    let records = sqlx::query!(r#"
        SELECT 
            source_token,
            source_chain_id,
            COUNT(*) as transaction_count,
            COALESCE(SUM(CASE WHEN source_amount != '' THEN source_amount::numeric ELSE 0 END), 0) as volume
        FROM intents 
        WHERE status = 'completed'
        GROUP BY source_token, source_chain_id 
        ORDER BY volume DESC 
        LIMIT 20
    "#)
    .fetch_all(db)
    .await?;
    
    let mut token_stats = Vec::new();
    
    for record in records {
        let token_address = record.source_token.parse::<ethers::types::Address>()
            .unwrap_or_default();
        
        let token_symbol = get_token_symbol(token_address); // Would fetch from cache or external API
        
        token_stats.push(crate::models::TokenStats {
            token_address,
            token_symbol,
            chain_id: record.source_chain_id as u64,
            volume: ethers::types::U256::from_dec_str(&record.volume.unwrap_or_default().to_string())
                .unwrap_or_default(),
            transaction_count: record.transaction_count.unwrap_or(0) as u64,
        });
    }
    
    Ok(token_stats)
}

// Get solver performance statistics
async fn get_solver_performance_stats(
    db: &sqlx::PgPool,
) -> Result<Vec<crate::models::SolverPerformance>> {
    let records = sqlx::query!(r#"
        SELECT 
            s.address,
            s.success_count,
            s.failure_count,
            s.reputation_score,
            s.total_volume,
            COALESCE(AVG(EXTRACT(EPOCH FROM (i.updated_at - i.created_at))), 0) as avg_execution_time
        FROM solvers s
        LEFT JOIN intents i ON i.solver_address = s.address AND i.status = 'completed'
        WHERE s.is_active = true
        GROUP BY s.address, s.success_count, s.failure_count, s.reputation_score, s.total_volume
        ORDER BY s.reputation_score DESC
        LIMIT 20
    "#)
    .fetch_all(db)
    .await?;
    
    let mut solver_performance = Vec::new();
    
    for record in records {
        let solver_address = record.address.parse::<ethers::types::Address>()
            .unwrap_or_default();
        
        solver_performance.push(crate::models::SolverPerformance {
            solver_address,
            success_count: record.success_count as u64,
            failure_count: record.failure_count as u64,
            average_execution_time: record.avg_execution_time.unwrap_or(0.0),
            reputation_score: record.reputation_score,
            total_volume: ethers::types::U256::from_dec_str(&record.total_volume)
                .unwrap_or_default(),
        });
    }
    
    Ok(solver_performance)
}

// Helper function to get chain name by ID
fn get_chain_name(chain_id: u64) -> String {
    match chain_id {
        1 => "Ethereum".to_string(),
        10 => "Optimism".to_string(),
        137 => "Polygon".to_string(),
        8453 => "Base".to_string(),
        42161 => "Arbitrum".to_string(),
        17000 => "Holesky".to_string(),
        _ => format!("Chain {}", chain_id),
    }
}

// Helper function to get token symbol (would typically fetch from cache or external API)
fn get_token_symbol(token_address: ethers::types::Address) -> String {
    // In a real implementation, this would:
    // 1. Check cache for known token symbols
    // 2. Query token contract for symbol
    // 3. Fall back to shortened address
    
    // For now, return shortened address
    let addr_str = format!("{:#x}", token_address);
    format!("{}...{}", &addr_str[0..6], &addr_str[addr_str.len()-4..])
}

// Initialize metrics on startup
pub fn initialize_metrics() {
    // Register custom metrics
    metrics::describe_gauge!("intents_total", "Total number of intents submitted");
    metrics::describe_gauge!("intents_successful", "Number of successfully executed intents");
    metrics::describe_gauge!("intents_failed", "Number of failed intents");
    metrics::describe_gauge!("intents_pending", "Number of pending intents");
    metrics::describe_gauge!("solvers_active", "Number of active solvers");
    metrics::describe_gauge!("intent_success_rate_percent", "Intent success rate as percentage");
    metrics::describe_gauge!("websocket_active_connections", "Number of active WebSocket connections");
    
    // Chain-specific metrics
    metrics::describe_gauge!("chain_intents_total", "Total intents per chain");
    metrics::describe_gauge!("chain_volume_total", "Total volume per chain");
    
    // Performance metrics
    metrics::describe_histogram!("intent_execution_duration_seconds", "Intent execution duration");
    metrics::describe_histogram!("api_request_duration_seconds", "API request duration");
    
    tracing::info!("Metrics initialized");
}

// Record intent execution metrics
pub fn record_intent_execution(duration: std::time::Duration, success: bool) {
    metrics::histogram!("intent_execution_duration_seconds").record(duration.as_secs_f64());
    
    if success {
        metrics::counter!("intents_executed_total", "status" => "success").increment(1);
    } else {
        metrics::counter!("intents_executed_total", "status" => "failure").increment(1);
    }
}

// Record API request metrics
pub fn record_api_request(endpoint: &str, method: &str, status: u16, duration: std::time::Duration) {
    metrics::histogram!(
        "api_request_duration_seconds",
        "endpoint" => endpoint.to_string(),
        "method" => method.to_string(),
        "status" => status.to_string()
    ).record(duration.as_secs_f64());
    
    metrics::counter!(
        "api_requests_total",
        "endpoint" => endpoint.to_string(),
        "method" => method.to_string(),
        "status" => status.to_string()
    ).increment(1);
}

// Record solver metrics
pub fn record_solver_activity(solver_address: ethers::types::Address, activity_type: &str) {
    metrics::counter!(
        "solver_activities_total",
        "solver" => format!("{:#x}", solver_address),
        "activity" => activity_type.to_string()
    ).increment(1);
}

// Record chain-specific metrics
pub fn record_chain_activity(chain_id: u64, activity_type: &str, value: f64) {
    metrics::gauge!(
        "chain_activity",
        "chain_id" => chain_id.to_string(),
        "activity" => activity_type.to_string()
    ).set(value);
}