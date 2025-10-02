use axum::{
    extract::{Query, State},
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::{
    models::{AppState, AnalyticsResponse, Claims},
    error::Result,
    metrics::generate_analytics_data,
    cache::CacheService,
};

// Analytics routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_analytics))
        .route("/public", get(get_public_analytics))
        .route("/chains", get(get_chain_analytics))
        .route("/tokens", get(get_token_analytics))
        .route("/solvers", get(get_solver_analytics))
        .route("/volume", get(get_volume_analytics))
}

// Main analytics endpoint (requires authentication)
async fn get_analytics(
    State(state): State<AppState>,
    _claims: Claims, // Require authentication
) -> Result<Json<AnalyticsResponse>> {
    let mut cache = CacheService::new(state.redis.clone());
    
    // Try cache first
    if let Some(cached_analytics) = cache.get_analytics().await? {
        return Ok(Json(cached_analytics));
    }
    
    // Generate fresh analytics
    let analytics = crate::metrics::generate_analytics_data(&state).await?;
    
    // Cache for future requests
    cache.cache_analytics(&analytics).await.ok();
    
    Ok(Json(analytics))
}

// Public analytics (limited data, no authentication required)
async fn get_public_analytics(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let mut cache = CacheService::new(state.redis.clone());
    
    // Check for cached public analytics
    if let Some(cached) = cache.get::<serde_json::Value>("public_analytics").await? {
        return Ok(Json(cached));
    }
    
    // Generate public analytics (limited data)
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
    
    let total_volume: Option<(String,)> = sqlx::query_as(
        "SELECT SUM(source_amount) FROM intents WHERE status = 'completed'"
    )
    .fetch_optional(&state.db)
    .await?;
    
    let volume_str = total_volume
        .and_then(|(sum,)| sum)
        .unwrap_or_else(|| "0".to_string());
    
    let public_data = serde_json::json!({
        "total_intents": total_intents.0,
        "successful_intents": successful_intents.0,
        "total_volume": volume_str,
        "success_rate": if total_intents.0 > 0 {
            successful_intents.0 as f64 / total_intents.0 as f64
        } else {
            0.0
        },
        "active_chains": state.config.chains.len(),
        "last_updated": chrono::Utc::now().to_rfc3339()
    });
    
    // Cache public analytics for 5 minutes
    cache.set("public_analytics", &public_data, Some(std::time::Duration::from_secs(300))).await.ok();
    
    Ok(Json(public_data))
}

// Chain-specific analytics
async fn get_chain_analytics(
    State(state): State<AppState>,
    Query(params): Query<ChainAnalyticsQuery>,
    _claims: Claims,
) -> Result<Json<serde_json::Value>> {
    let chain_filter = if let Some(chain_id) = params.chain_id {
        format!("AND source_chain_id = {}", chain_id)
    } else {
        String::new()
    };
    
    let query = format!(r#"
        SELECT 
            source_chain_id as chain_id,
            COUNT(*) as total_intents,
            COUNT(CASE WHEN status = 'completed' THEN 1 END) as successful_intents,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_intents,
            COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_intents,
            COALESCE(SUM(CASE WHEN status = 'completed' AND source_amount != '' THEN source_amount::numeric ELSE 0 END), 0) as total_volume
        FROM intents 
        WHERE created_at >= NOW() - INTERVAL '{}' {}
        GROUP BY source_chain_id
        ORDER BY total_volume DESC
    ", params.timeframe.unwrap_or_else(|| "24 hours".to_string()), chain_filter);
    
    let records = sqlx::query(&query)
        .fetch_all(&state.db)
        .await?;
    
    let mut chain_analytics = Vec::new();
    
    for record in records {
        let chain_id: i64 = record.get("chain_id");
        let total_intents: i64 = record.get("total_intents");
        let successful_intents: i64 = record.get("successful_intents");
        let failed_intents: i64 = record.get("failed_intents");
        let pending_intents: i64 = record.get("pending_intents");
        let total_volume: sqlx::types::BigDecimal = record.get("total_volume");
        
        chain_analytics.push(serde_json::json!({
            "chain_id": chain_id,
            "chain_name": get_chain_name(chain_id as u64),
            "total_intents": total_intents,
            "successful_intents": successful_intents,
            "failed_intents": failed_intents,
            "pending_intents": pending_intents,
            "total_volume": total_volume.to_string(),
            "success_rate": if total_intents > 0 {
                successful_intents as f64 / total_intents as f64
            } else {
                0.0
            }
        }));
    }
    
    Ok(Json(serde_json::json!({
        "chains": chain_analytics,
        "timeframe": params.timeframe.unwrap_or_else(|| "24 hours".to_string()),
        "generated_at": chrono::Utc::now().to_rfc3339()
    })))
}

// Token analytics
async fn get_token_analytics(
    State(state): State<AppState>,
    Query(params): Query<TokenAnalyticsQuery>,
    _claims: Claims,
) -> Result<Json<serde_json::Value>> {
    let limit = params.limit.unwrap_or(20).min(100);
    let timeframe = params.timeframe.unwrap_or_else(|| "24 hours".to_string());
    
    let records = sqlx::query!(r#"
        SELECT 
            source_token,
            source_chain_id,
            COUNT(*) as transaction_count,
            COALESCE(SUM(CASE WHEN status = 'completed' AND source_amount != '' THEN source_amount::numeric ELSE 0 END), 0) as volume,
            AVG(CASE WHEN status = 'completed' THEN 1.0 ELSE 0.0 END) as success_rate
        FROM intents 
        WHERE created_at >= NOW() - INTERVAL $1
        GROUP BY source_token, source_chain_id 
        ORDER BY volume DESC 
        LIMIT $2
    "#, timeframe, limit as i64)
    .fetch_all(&state.db)
    .await?;
    
    let mut token_analytics = Vec::new();
    
    for record in records {
        token_analytics.push(serde_json::json!({
            "token_address": record.source_token,
            "chain_id": record.source_chain_id,
            "chain_name": get_chain_name(record.source_chain_id as u64),
            "transaction_count": record.transaction_count.unwrap_or(0),
            "volume": record.volume.unwrap_or_default().to_string(),
            "success_rate": record.success_rate.unwrap_or(0.0)
        }));
    }
    
    Ok(Json(serde_json::json!({
        "tokens": token_analytics,
        "timeframe": timeframe,
        "limit": limit,
        "generated_at": chrono::Utc::now().to_rfc3339()
    })))
}

// Solver analytics
async fn get_solver_analytics(
    State(state): State<AppState>,
    Query(params): Query<SolverAnalyticsQuery>,
    _claims: Claims,
) -> Result<Json<serde_json::Value>> {
    let limit = params.limit.unwrap_or(20).min(100);
    
    let records = sqlx::query!(r#"
        SELECT 
            s.address,
            s.success_count,
            s.failure_count,
            s.reputation_score,
            s.total_volume,
            s.fee_rate,
            s.is_active,
            COUNT(i.id) as recent_intents,
            AVG(CASE WHEN i.status = 'completed' THEN EXTRACT(EPOCH FROM (i.updated_at - i.created_at)) END) as avg_execution_time
        FROM solvers s
        LEFT JOIN intents i ON i.solver_address = s.address AND i.created_at >= NOW() - INTERVAL '24 hours'
        WHERE s.is_active = true
        GROUP BY s.address, s.success_count, s.failure_count, s.reputation_score, s.total_volume, s.fee_rate, s.is_active
        ORDER BY s.reputation_score DESC
        LIMIT $1
    "#, limit as i64)
    .fetch_all(&state.db)
    .await?;
    
    let mut solver_analytics = Vec::new();
    
    for record in records {
        solver_analytics.push(serde_json::json!({
            "address": record.address,
            "success_count": record.success_count,
            "failure_count": record.failure_count,
            "reputation_score": record.reputation_score,
            "total_volume": record.total_volume,
            "fee_rate": record.fee_rate,
            "is_active": record.is_active,
            "recent_intents_24h": record.recent_intents.unwrap_or(0),
            "avg_execution_time": record.avg_execution_time.unwrap_or(0.0),
            "success_rate": if record.success_count + record.failure_count > 0 {
                record.success_count as f64 / (record.success_count + record.failure_count) as f64
            } else {
                0.0
            }
        }));
    }
    
    Ok(Json(serde_json::json!({
        "solvers": solver_analytics,
        "limit": limit,
        "generated_at": chrono::Utc::now().to_rfc3339()
    })))
}

// Volume analytics over time
async fn get_volume_analytics(
    State(state): State<AppState>,
    Query(params): Query<VolumeAnalyticsQuery>,
    _claims: Claims,
) -> Result<Json<serde_json::Value>> {
    let timeframe = params.timeframe.unwrap_or_else(|| "7 days".to_string());
    let interval = params.interval.unwrap_or_else(|| "1 hour".to_string());
    
    let records = sqlx::query!(r#"
        SELECT 
            date_trunc($2, created_at) as time_bucket,
            COUNT(*) as intent_count,
            COUNT(CASE WHEN status = 'completed' THEN 1 END) as successful_count,
            COALESCE(SUM(CASE WHEN status = 'completed' AND source_amount != '' THEN source_amount::numeric ELSE 0 END), 0) as volume
        FROM intents 
        WHERE created_at >= NOW() - INTERVAL $1
        GROUP BY time_bucket
        ORDER BY time_bucket
    "#, timeframe, interval)
    .fetch_all(&state.db)
    .await?;
    
    let mut volume_data = Vec::new();
    
    for record in records {
        volume_data.push(serde_json::json!({
            "timestamp": record.time_bucket.unwrap_or_default().to_rfc3339(),
            "intent_count": record.intent_count.unwrap_or(0),
            "successful_count": record.successful_count.unwrap_or(0),
            "volume": record.volume.unwrap_or_default().to_string()
        }));
    }
    
    Ok(Json(serde_json::json!({
        "volume_data": volume_data,
        "timeframe": timeframe,
        "interval": interval,
        "generated_at": chrono::Utc::now().to_rfc3339()
    })))
}

// Query parameter structs
#[derive(Deserialize)]
struct ChainAnalyticsQuery {
    chain_id: Option<u64>,
    timeframe: Option<String>,
}

#[derive(Deserialize)]
struct TokenAnalyticsQuery {
    limit: Option<u64>,
    timeframe: Option<String>,
}

#[derive(Deserialize)]
struct SolverAnalyticsQuery {
    limit: Option<u64>,
}

#[derive(Deserialize)]
struct VolumeAnalyticsQuery {
    timeframe: Option<String>,
    interval: Option<String>,
}

// Helper function
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