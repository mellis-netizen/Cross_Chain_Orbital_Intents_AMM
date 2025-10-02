use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use ethers::types::{Address, H256};
use std::str::FromStr;

use crate::{
    models::*,
    database::{IntentDb, intent_record_to_response},
    cache::CacheService,
    error::{Result, validation_error, not_found},
    auth::{extract_user_address, check_permission},
    websocket::broadcast_intent_update,
};

// Intent routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(submit_intent))
        .route("/", get(get_user_intents))
        .route("/:intent_id", get(get_intent_by_id))
        .route("/:intent_id/status", get(get_intent_status))
        .route("/:intent_id/cancel", post(cancel_intent))
        .route("/pending", get(get_pending_intents))
}

// Submit a new intent
async fn submit_intent(
    State(state): State<AppState>,
    Json(request): Json<SubmitIntentRequest>,
) -> Result<Json<IntentResponse>> {
    // Validate request
    validate_submit_intent_request(&request)?;
    
    // Submit to intent engine
    let intent_id = state.intents_engine
        .submit_intent(convert_to_engine_intent(&request))
        .await
        .map_err(|e| crate::error::ApiError::IntentEngine(e.to_string()))?;
    
    // Store in database
    let record = IntentDb::insert_intent(&state.db, intent_id, &request).await?;
    
    // Cache the intent status
    let mut cache = CacheService::new(state.redis.clone());
    let status_response = IntentStatusResponse {
        intent_id,
        status: "pending".to_string(),
        progress: IntentProgress {
            current_step: "Submitted".to_string(),
            steps_completed: 1,
            total_steps: 5,
            percentage: 20.0,
        },
        estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        error_message: None,
    };
    
    cache.cache_intent_status(intent_id, &status_response).await.ok();
    
    // Convert to response
    let response = intent_record_to_response(record)?;
    
    // Broadcast update via WebSocket
    let update_msg = IntentUpdateMessage {
        intent_id,
        status: "pending".to_string(),
        progress: status_response.progress,
        details: Some(serde_json::json!({
            "source_chain": request.source_chain_id,
            "dest_chain": request.dest_chain_id,
            "amount": request.source_amount.to_string()
        })),
    };
    
    broadcast_intent_update(intent_id, update_msg).await;
    
    tracing::info!(
        "Intent submitted: {:#x} from user {:#x}",
        intent_id,
        request.user_address
    );
    
    Ok(Json(response))
}

// Get intents for authenticated user
async fn get_user_intents(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    claims: Claims,
) -> Result<Json<PaginatedResponse<IntentResponse>>> {
    let user_address = extract_user_address(&claims)?;
    
    // Get intents from database
    let (records, total_count) = IntentDb::get_intents_by_user(
        &state.db,
        user_address,
        &pagination,
    ).await?;
    
    // Convert to responses
    let intents: Result<Vec<IntentResponse>> = records
        .into_iter()
        .map(intent_record_to_response)
        .collect();
    
    let pagination_meta = PaginationMeta {
        current_page: pagination.page.unwrap_or(1),
        per_page: pagination.limit.unwrap_or(20),
        total_items: total_count,
        total_pages: (total_count + pagination.limit.unwrap_or(20) - 1) / pagination.limit.unwrap_or(20),
        has_next: pagination.page.unwrap_or(1) * pagination.limit.unwrap_or(20) < total_count,
        has_prev: pagination.page.unwrap_or(1) > 1,
    };
    
    Ok(Json(PaginatedResponse {
        data: intents?,
        pagination: pagination_meta,
    }))
}

// Get specific intent by ID
async fn get_intent_by_id(
    State(state): State<AppState>,
    Path(intent_id_str): Path<String>,
    claims: Option<Claims>,
) -> Result<Json<IntentResponse>> {
    let intent_id = H256::from_str(&intent_id_str)
        .map_err(|_| validation_error("Invalid intent ID format"))?;
    
    // Get intent from database
    let record = IntentDb::get_intent_by_id(&state.db, intent_id)
        .await?
        .ok_or_else(|| not_found("Intent"))?;
    
    // Check permissions - users can only see their own intents unless public
    if let Some(claims) = claims {
        let user_address = extract_user_address(&claims)?;
        let intent_user = Address::from_str(&record.user_address)
            .map_err(|_| crate::error::internal_error("Invalid user address in database"))?;
        
        if user_address != intent_user {
            check_permission(&claims, "/api/v1/intents/*", "GET")?;
        }
    }
    
    let response = intent_record_to_response(record)?;
    Ok(Json(response))
}

// Get intent status (with caching)
async fn get_intent_status(
    State(state): State<AppState>,
    Path(intent_id_str): Path<String>,
) -> Result<Json<IntentStatusResponse>> {
    let intent_id = H256::from_str(&intent_id_str)
        .map_err(|_| validation_error("Invalid intent ID format"))?;
    
    // Try cache first
    let mut cache = CacheService::new(state.redis.clone());
    if let Some(cached_status) = cache.get_intent_status(intent_id).await? {
        return Ok(Json(cached_status));
    }
    
    // Get from database
    let record = IntentDb::get_intent_by_id(&state.db, intent_id)
        .await?
        .ok_or_else(|| not_found("Intent"))?;
    
    // Get live status from intent engine
    let engine_status = state.intents_engine
        .get_intent_status(intent_id)
        .await
        .map_err(|e| crate::error::ApiError::IntentEngine(e.to_string()))?;
    
    // Convert to response
    let status_response = convert_engine_status_to_response(intent_id, &engine_status, &record)?;
    
    // Cache for future requests
    cache.cache_intent_status(intent_id, &status_response).await.ok();
    
    Ok(Json(status_response))
}

// Cancel an intent
async fn cancel_intent(
    State(state): State<AppState>,
    Path(intent_id_str): Path<String>,
    claims: Claims,
) -> Result<StatusCode> {
    let intent_id = H256::from_str(&intent_id_str)
        .map_err(|_| validation_error("Invalid intent ID format"))?;
    
    let user_address = extract_user_address(&claims)?;
    
    // Verify ownership
    let record = IntentDb::get_intent_by_id(&state.db, intent_id)
        .await?
        .ok_or_else(|| not_found("Intent"))?;
    
    let intent_user = Address::from_str(&record.user_address)
        .map_err(|_| crate::error::internal_error("Invalid user address in database"))?;
    
    if user_address != intent_user {
        return Err(crate::error::ApiError::Authorization("Not authorized to cancel this intent".to_string()));
    }
    
    // Check if intent can be cancelled
    if !matches!(record.status.as_str(), "pending" | "matched") {
        return Err(crate::error::ApiError::BadRequest(
            "Intent cannot be cancelled in current status".to_string()
        ));
    }
    
    // Cancel in intent engine
    // TODO: Implement cancel functionality in intent engine
    
    // Update database
    IntentDb::update_intent_status(
        &state.db,
        intent_id,
        "cancelled",
        None,
        None,
        None,
        None,
        None,
        Some("Cancelled by user".to_string()),
    ).await?;
    
    // Broadcast update
    let update_msg = IntentUpdateMessage {
        intent_id,
        status: "cancelled".to_string(),
        progress: IntentProgress {
            current_step: "Cancelled".to_string(),
            steps_completed: 5,
            total_steps: 5,
            percentage: 100.0,
        },
        details: Some(serde_json::json!({
            "cancelled_by": "user",
            "reason": "User requested cancellation"
        })),
    };
    
    broadcast_intent_update(intent_id, update_msg).await;
    
    tracing::info!(
        "Intent cancelled: {:#x} by user {:#x}",
        intent_id,
        user_address
    );
    
    Ok(StatusCode::OK)
}

// Get pending intents (for solvers)
async fn get_pending_intents(
    State(state): State<AppState>,
    Query(params): Query<PendingIntentsQuery>,
    claims: Claims,
) -> Result<Json<Vec<IntentResponse>>> {
    // Check if user is a solver
    check_permission(&claims, "/api/v1/intents/pending", "GET")?;
    
    let limit = params.limit.unwrap_or(50).min(100); // Max 100 intents
    
    // Try cache first
    let mut cache = CacheService::new(state.redis.clone());
    if let Some(cached_intent_ids) = cache.get_pending_intents().await? {
        if cached_intent_ids.len() <= limit as usize {
            // Get full intent data
            let mut intents = Vec::new();
            for intent_id in cached_intent_ids {
                if let Some(record) = IntentDb::get_intent_by_id(&state.db, intent_id).await? {
                    intents.push(intent_record_to_response(record)?);
                }
            }
            return Ok(Json(intents));
        }
    }
    
    // Get from database
    let records = IntentDb::get_pending_intents(&state.db, limit).await?;
    
    // Cache the intent IDs
    let intent_ids: Vec<H256> = records.iter()
        .filter_map(|r| H256::from_str(&r.intent_id).ok())
        .collect();
    cache.cache_pending_intents(&intent_ids).await.ok();
    
    // Convert to responses
    let intents: Result<Vec<IntentResponse>> = records
        .into_iter()
        .map(intent_record_to_response)
        .collect();
    
    Ok(Json(intents?))
}

// Helper functions
fn validate_submit_intent_request(request: &SubmitIntentRequest) -> Result<()> {
    // Basic validation
    if request.source_chain_id == request.dest_chain_id {
        return Err(validation_error("Source and destination chains cannot be the same"));
    }
    
    if request.source_amount.is_zero() {
        return Err(validation_error("Source amount must be greater than zero"));
    }
    
    if request.min_dest_amount.is_zero() {
        return Err(validation_error("Minimum destination amount must be greater than zero"));
    }
    
    if request.deadline <= chrono::Utc::now() {
        return Err(validation_error("Deadline must be in the future"));
    }
    
    // Validate slippage tolerance
    if let Some(slippage) = request.slippage_tolerance {
        if slippage < 0.0 || slippage > 0.5 {
            return Err(validation_error("Slippage tolerance must be between 0% and 50%"));
        }
    }
    
    Ok(())
}

fn convert_to_engine_intent(request: &SubmitIntentRequest) -> intents_engine::intent::Intent {
    // Convert API request to engine intent
    // This would use the actual Intent struct from the engine
    todo!("Implement conversion to engine intent")
}

fn convert_engine_status_to_response(
    intent_id: H256,
    engine_status: &intents_engine::intent::IntentStatus,
    _record: &IntentRecord,
) -> Result<IntentStatusResponse> {
    // Convert engine status to API response
    // This would use the actual IntentStatus from the engine
    Ok(IntentStatusResponse {
        intent_id,
        status: "pending".to_string(), // TODO: Map from engine status
        progress: IntentProgress {
            current_step: "Processing".to_string(),
            steps_completed: 2,
            total_steps: 5,
            percentage: 40.0,
        },
        estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(3)),
        error_message: None,
    })
}

// Query parameters for pending intents
#[derive(serde::Deserialize)]
struct PendingIntentsQuery {
    limit: Option<u64>,
    chain_id: Option<u64>,
}