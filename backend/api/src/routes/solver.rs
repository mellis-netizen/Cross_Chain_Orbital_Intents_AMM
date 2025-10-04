use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use ethers::types::{Address, U256};
use std::str::FromStr;

use crate::{
    models::*,
    database::{SolverDb, solver_record_to_response},
    cache::CacheService,
    error::{Result, validation_error, not_found},
    auth::{extract_user_address, check_permission},
    crypto::{
        verify_signature, 
        create_solver_registration_message,
        create_secure_solver_registration_message,
        verify_message_freshness,
        SignatureRateLimiter,
        generate_secure_nonce
    },
};

// Solver routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_solver))
        .route("/register/message", post(get_registration_message))
        .route("/", get(get_solvers))
        .route("/:address", get(get_solver_by_address))
        .route("/:address/performance", get(get_solver_performance))
        .route("/:address/update", put(update_solver))
        .route("/:address/deactivate", post(deactivate_solver))
        .route("/leaderboard", get(get_solver_leaderboard))
}

// Register a new solver with enhanced security
async fn register_solver(
    State(state): State<AppState>,
    Json(request): Json<SolverRegistrationRequest>,
) -> Result<Json<SolverResponse>> {
    // Rate limiting for registration attempts
    let rate_limiter = SignatureRateLimiter::new(5, 300); // 5 attempts per 5 minutes
    let client_ip = "127.0.0.1"; // TODO: Extract from request headers
    rate_limiter.check_rate_limit(&format!("register:{}", client_ip))?;
    
    // Enhanced validation
    validate_solver_registration_enhanced(&request)?;
    
    // Check for replay attacks using nonce and timestamp if available
    if let Some(timestamp) = extract_timestamp_from_request(&request) {
        verify_message_freshness(timestamp, 300)?; // 5 minute tolerance
    }
    
    // Create the message that should have been signed
    let message = create_solver_registration_message(
        request.solver_address,
        &request.bond_amount,
        &request.supported_chains,
        request.fee_rate,
    );
    
    // Enhanced signature verification with dual verification
    let is_valid_signature = verify_signature(
        &message,
        &request.signature,
        request.solver_address,
    )?;
    
    if !is_valid_signature {
        // Log suspicious activity
        tracing::warn!(
            "Invalid signature attempt for solver registration: {:#x} from IP: {}",
            request.solver_address,
            client_ip
        );
        return Err(crate::error::ApiError::Authorization(
            "Invalid signature for solver registration".to_string()
        ));
    }
    
    // Additional security checks
    perform_solver_security_checks(&request, &state).await?;
    
    // Check if solver already exists
    if let Some(_existing) = SolverDb::get_solver_by_address(&state.db, request.solver_address).await? {
        return Err(crate::error::ApiError::Conflict("Solver already registered".to_string()));
    }
    
    // Register solver in database with audit trail
    let record = SolverDb::register_solver(&state.db, &request).await?;
    
    // Log successful registration
    tracing::info!(
        "Solver registered successfully: {:#x} with bond: {} wei, chains: {:?}",
        request.solver_address,
        request.bond_amount,
        request.supported_chains
    );
    
    // Update cache
    let mut cache = CacheService::new(state.redis.clone());
    cache.cache_solver_reputation(request.solver_address, record.reputation_score).await.ok();
    
    let response = solver_record_to_response(record)?;
    
    Ok(Json(response))
}

// Get the message that needs to be signed for solver registration
async fn get_registration_message(
    Json(request): Json<RegistrationMessageRequest>,
) -> Result<Json<RegistrationMessageResponse>> {
    // Validate basic parameters
    if request.bond_amount.is_zero() {
        return Err(validation_error("Bond amount must be greater than zero"));
    }
    
    if request.supported_chains.is_empty() {
        return Err(validation_error("Must support at least one chain"));
    }
    
    if request.fee_rate < 0.0 || request.fee_rate > 1000.0 {
        return Err(validation_error("Fee rate must be between 0 and 1000 basis points"));
    }
    
    // Create the message to be signed
    let message = create_solver_registration_message(
        request.solver_address,
        &request.bond_amount,
        &request.supported_chains,
        request.fee_rate,
    );
    
    let message_str = String::from_utf8(message)
        .map_err(|_| crate::error::ApiError::Internal("Failed to convert message to string".to_string()))?;
    
    Ok(Json(RegistrationMessageResponse {
        message: message_str,
        solver_address: request.solver_address,
        bond_amount: request.bond_amount,
        supported_chains: request.supported_chains,
        fee_rate: request.fee_rate,
    }))
}

// Get list of solvers with filtering
async fn get_solvers(
    State(state): State<AppState>,
    Query(params): Query<SolverQuery>,
) -> Result<Json<PaginatedResponse<SolverResponse>>> {
    let solvers = SolverDb::get_active_solvers(&state.db, params.chain_id).await?;
    
    // Apply filters
    let mut filtered_solvers = solvers;
    
    if let Some(min_reputation) = params.min_reputation {
        filtered_solvers.retain(|s| s.reputation_score >= min_reputation);
    }
    
    if let Some(max_fee_rate) = params.max_fee_rate {
        filtered_solvers.retain(|s| s.fee_rate <= max_fee_rate);
    }
    
    // Sort by reputation score
    filtered_solvers.sort_by(|a, b| b.reputation_score.partial_cmp(&a.reputation_score).unwrap());
    
    // Apply pagination
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = (page - 1) * limit;
    
    let total_count = filtered_solvers.len() as u64;
    let paginated_solvers = filtered_solvers
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect::<Vec<_>>();
    
    // Convert to responses
    let solver_responses: Result<Vec<SolverResponse>> = paginated_solvers
        .into_iter()
        .map(solver_record_to_response)
        .collect();
    
    let pagination_meta = PaginationMeta {
        current_page: page,
        per_page: limit,
        total_items: total_count,
        total_pages: (total_count + limit - 1) / limit,
        has_next: page * limit < total_count,
        has_prev: page > 1,
    };
    
    Ok(Json(PaginatedResponse {
        data: solver_responses?,
        pagination: pagination_meta,
    }))
}

// Get specific solver by address
async fn get_solver_by_address(
    State(state): State<AppState>,
    Path(address_str): Path<String>,
) -> Result<Json<SolverResponse>> {
    let address = Address::from_str(&address_str)
        .map_err(|_| validation_error("Invalid solver address format"))?;
    
    let record = SolverDb::get_solver_by_address(&state.db, address)
        .await?
        .ok_or_else(|| not_found("Solver"))?;
    
    let response = solver_record_to_response(record)?;
    Ok(Json(response))
}

// Get solver performance metrics
async fn get_solver_performance(
    State(state): State<AppState>,
    Path(address_str): Path<String>,
    Query(params): Query<PerformanceQuery>,
) -> Result<Json<SolverPerformance>> {
    let address = Address::from_str(&address_str)
        .map_err(|_| validation_error("Invalid solver address format"))?;
    
    // Check cache first
    let mut cache = CacheService::new(state.redis.clone());
    if let Some(cached_performance) = cache.get_solver_performance(address).await? {
        return Ok(Json(cached_performance));
    }
    
    // Get from database
    let timeframe = params.timeframe.unwrap_or_else(|| "30 days".to_string());
    
    let performance_data = sqlx::query!(r#"
        SELECT 
            COUNT(*) as total_intents,
            COUNT(CASE WHEN status = 'completed' THEN 1 END) as successful_intents,
            COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_intents,
            AVG(CASE WHEN status = 'completed' THEN EXTRACT(EPOCH FROM (updated_at - created_at)) END) as avg_execution_time,
            COALESCE(SUM(CASE WHEN status = 'completed' AND source_amount != '' THEN source_amount::numeric ELSE 0 END), 0) as total_volume
        FROM intents 
        WHERE solver_address = $1 AND created_at >= NOW() - INTERVAL $2
    "#, format!("{:#x}", address), timeframe)
    .fetch_one(&state.db)
    .await?;
    
    // Get solver reputation
    let solver = SolverDb::get_solver_by_address(&state.db, address)
        .await?
        .ok_or_else(|| not_found("Solver"))?;
    
    let performance = SolverPerformance {
        solver_address: address,
        success_count: performance_data.successful_intents.unwrap_or(0) as u64,
        failure_count: performance_data.failed_intents.unwrap_or(0) as u64,
        average_execution_time: performance_data.avg_execution_time.unwrap_or(0.0),
        reputation_score: solver.reputation_score,
        total_volume: crate::database::string_to_u256(&performance_data.total_volume.to_string())?,
    };
    
    // Cache the performance data
    cache.cache_solver_performance(address, &performance).await.ok();
    
    Ok(Json(performance))
}

// Update solver settings (only by the solver themselves)
async fn update_solver(
    State(state): State<AppState>,
    Path(address_str): Path<String>,
    claims: Claims,
    Json(update_request): Json<SolverUpdateRequest>,
) -> Result<StatusCode> {
    let address = Address::from_str(&address_str)
        .map_err(|_| validation_error("Invalid solver address format"))?;
    
    let user_address = extract_user_address(&claims)?;
    
    // Verify that the user is updating their own solver
    if user_address != address {
        return Err(crate::error::ApiError::Authorization(
            "Can only update your own solver settings".to_string()
        ));
    }
    
    // Verify solver exists
    let _solver = SolverDb::get_solver_by_address(&state.db, address)
        .await?
        .ok_or_else(|| not_found("Solver"))?;
    
    // Update solver settings
    sqlx::query!(r#"
        UPDATE solvers SET 
            fee_rate = COALESCE($2, fee_rate),
            contact_info = COALESCE($3, contact_info),
            is_active = COALESCE($4, is_active)
        WHERE address = $1
    "#,
        format!("{:#x}", address),
        update_request.fee_rate,
        update_request.contact_info,
        update_request.is_active
    )
    .execute(&state.db)
    .await?;
    
    // Invalidate cache
    let mut cache = CacheService::new(state.redis.clone());
    cache.delete(&crate::cache::CacheKeys::solver_reputation(address)).await.ok();
    cache.delete(&crate::cache::CacheKeys::solver_performance(address)).await.ok();
    
    tracing::info!("Solver updated: {:#x}", address);
    
    Ok(StatusCode::OK)
}

// Deactivate solver (admin only)
async fn deactivate_solver(
    State(state): State<AppState>,
    Path(address_str): Path<String>,
    claims: Claims,
    Json(deactivate_request): Json<DeactivateSolverRequest>,
) -> Result<StatusCode> {
    let address = Address::from_str(&address_str)
        .map_err(|_| validation_error("Invalid solver address format"))?;
    
    // Check admin permissions
    check_permission(&claims, "/api/v1/solver/*/deactivate", "POST")?;
    
    // Deactivate or slash solver
    if deactivate_request.slash {
        SolverDb::slash_solver(&state.db, address, &deactivate_request.reason).await?;
        tracing::warn!("Solver slashed: {:#x} - {}", address, deactivate_request.reason);
    } else {
        sqlx::query!("UPDATE solvers SET is_active = false WHERE address = $1", 
                    format!("{:#x}", address))
            .execute(&state.db)
            .await?;
        tracing::info!("Solver deactivated: {:#x}", address);
    }
    
    // Invalidate cache
    let mut cache = CacheService::new(state.redis.clone());
    cache.delete(&crate::cache::CacheKeys::solver_reputation(address)).await.ok();
    
    Ok(StatusCode::OK)
}

// Get solver leaderboard
async fn get_solver_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<serde_json::Value>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let timeframe = params.timeframe.unwrap_or_else(|| "30 days".to_string());
    
    let leaderboard_data = sqlx::query!(r#"
        SELECT 
            s.address,
            s.reputation_score,
            s.success_count,
            s.failure_count,
            s.total_volume,
            COUNT(i.id) as recent_intents,
            AVG(CASE WHEN i.status = 'completed' THEN EXTRACT(EPOCH FROM (i.updated_at - i.created_at)) END) as avg_execution_time
        FROM solvers s
        LEFT JOIN intents i ON i.solver_address = s.address AND i.created_at >= NOW() - INTERVAL $2
        WHERE s.is_active = true AND s.is_slashed = false
        GROUP BY s.address, s.reputation_score, s.success_count, s.failure_count, s.total_volume
        ORDER BY s.reputation_score DESC, s.total_volume DESC
        LIMIT $1
    "#, limit as i64, timeframe)
    .fetch_all(&state.db)
    .await?;
    
    let mut leaderboard = Vec::new();
    
    for (rank, record) in leaderboard_data.iter().enumerate() {
        leaderboard.push(serde_json::json!({
            "rank": rank + 1,
            "address": record.address,
            "reputation_score": record.reputation_score,
            "success_count": record.success_count,
            "failure_count": record.failure_count,
            "total_volume": record.total_volume,
            "recent_intents": record.recent_intents.unwrap_or(0),
            "avg_execution_time": record.avg_execution_time.unwrap_or(0.0),
            "success_rate": if record.success_count + record.failure_count > 0 {
                record.success_count as f64 / (record.success_count + record.failure_count) as f64
            } else {
                0.0
            }
        }));
    }
    
    Ok(Json(serde_json::json!({
        "leaderboard": leaderboard,
        "timeframe": timeframe,
        "limit": limit,
        "generated_at": chrono::Utc::now().to_rfc3339()
    })))
}

// Helper functions and validation
fn validate_solver_registration(request: &SolverRegistrationRequest) -> Result<()> {
    if request.bond_amount.is_zero() {
        return Err(validation_error("Bond amount must be greater than zero"));
    }
    
    if request.supported_chains.is_empty() {
        return Err(validation_error("Must support at least one chain"));
    }
    
    if request.fee_rate < 0.0 || request.fee_rate > 1000.0 {
        return Err(validation_error("Fee rate must be between 0 and 1000 basis points"));
    }
    
    Ok(())
}

/// Enhanced security validation for solver registration
fn validate_solver_registration_enhanced(request: &SolverRegistrationRequest) -> Result<()> {
    // Basic validation
    validate_solver_registration(request)?;
    
    // Additional security checks
    
    // Validate address format and ensure it's not zero
    if request.solver_address.is_zero() {
        return Err(validation_error("Solver address cannot be zero"));
    }
    
    // Validate signature format and length
    let sig_without_prefix = request.signature.strip_prefix("0x").unwrap_or(&request.signature);
    if sig_without_prefix.len() != 130 {
        return Err(validation_error("Invalid signature format"));
    }
    
    // Validate hex format
    if hex::decode(sig_without_prefix).is_err() {
        return Err(validation_error("Signature must be valid hexadecimal"));
    }
    
    // Validate bond amount reasonable bounds (prevent overflow/underflow attacks)
    let min_bond = U256::from(10_000_000_000_000_000u64); // 0.01 ETH minimum
    let max_bond = U256::from(1000u64) * U256::from(10u64).pow(18.into()); // 1000 ETH maximum
    
    if request.bond_amount < min_bond {
        return Err(validation_error("Bond amount too low (minimum 0.01 ETH)"));
    }
    
    if request.bond_amount > max_bond {
        return Err(validation_error("Bond amount too high (maximum 1000 ETH)"));
    }
    
    // Validate supported chains
    if request.supported_chains.len() > 50 {
        return Err(validation_error("Too many supported chains (maximum 50)"));
    }
    
    for &chain_id in &request.supported_chains {
        if chain_id == 0 {
            return Err(validation_error("Invalid chain ID: 0"));
        }
        // Add specific chain validation if needed
    }
    
    // Validate fee rate with more precision
    if request.fee_rate < 0.0 || request.fee_rate > 10000.0 {
        return Err(validation_error("Fee rate must be between 0 and 10000 basis points"));
    }
    
    // Ensure fee rate has reasonable precision (prevent float precision attacks)
    if (request.fee_rate * 100.0).fract() != 0.0 {
        return Err(validation_error("Fee rate precision too high (max 2 decimal places)"));
    }
    
    Ok(())
}

/// Extract timestamp from request if embedded in signature or metadata
fn extract_timestamp_from_request(_request: &SolverRegistrationRequest) -> Option<u64> {
    // TODO: Implement timestamp extraction from request metadata
    // For now, we'll use current time as a placeholder
    Some(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}

/// Perform additional security checks on solver registration
async fn perform_solver_security_checks(
    request: &SolverRegistrationRequest,
    state: &AppState,
) -> Result<()> {
    // Check for blacklisted addresses
    if is_address_blacklisted(request.solver_address, state).await? {
        return Err(crate::error::ApiError::Authorization(
            "Address is blacklisted".to_string()
        ));
    }
    
    // Check for suspicious patterns
    if detect_suspicious_registration_pattern(request, state).await? {
        tracing::warn!(
            "Suspicious registration pattern detected for address: {:#x}",
            request.solver_address
        );
        // Could implement additional verification steps here
    }
    
    // Validate against known compromised addresses
    if is_address_compromised(request.solver_address).await? {
        return Err(crate::error::ApiError::Authorization(
            "Address appears to be compromised".to_string()
        ));
    }
    
    Ok(())
}

/// Check if an address is blacklisted
async fn is_address_blacklisted(address: Address, _state: &AppState) -> Result<bool> {
    // TODO: Implement blacklist checking against database or external service
    // For now, we'll check against a static list of known bad addresses
    let blacklisted_addresses = vec![
        // Add known malicious addresses here
        Address::zero(),
    ];
    
    Ok(blacklisted_addresses.contains(&address))
}

/// Detect suspicious registration patterns
async fn detect_suspicious_registration_pattern(
    request: &SolverRegistrationRequest,
    state: &AppState,
) -> Result<bool> {
    // Check for rapid registrations from similar addresses
    let similar_recent_registrations = sqlx::query!(
        r#"
        SELECT COUNT(*) as count 
        FROM solvers 
        WHERE registered_at > NOW() - INTERVAL '1 hour'
        AND bond_amount = $1
        "#,
        request.bond_amount.to_string()
    )
    .fetch_one(&state.db)
    .await?;
    
    // If more than 5 registrations with same bond amount in last hour, flag as suspicious
    Ok(similar_recent_registrations.count.unwrap_or(0) > 5)
}

/// Check if an address is known to be compromised
async fn is_address_compromised(_address: Address) -> Result<bool> {
    // TODO: Integrate with external threat intelligence services
    // For now, return false
    Ok(false)
}

// Request/Query structs
#[derive(serde::Deserialize)]
struct SolverQuery {
    chain_id: Option<u64>,
    min_reputation: Option<f64>,
    max_fee_rate: Option<f64>,
    page: Option<u64>,
    limit: Option<u64>,
}

#[derive(serde::Deserialize)]
struct PerformanceQuery {
    timeframe: Option<String>,
}

#[derive(serde::Deserialize)]
struct SolverUpdateRequest {
    fee_rate: Option<f64>,
    contact_info: Option<String>,
    is_active: Option<bool>,
}

#[derive(serde::Deserialize)]
struct DeactivateSolverRequest {
    reason: String,
    slash: bool,
}

#[derive(serde::Deserialize)]
struct LeaderboardQuery {
    limit: Option<u64>,
    timeframe: Option<String>,
}

#[derive(serde::Deserialize)]
struct RegistrationMessageRequest {
    solver_address: Address,
    bond_amount: U256,
    supported_chains: Vec<u64>,
    fee_rate: f64,
}

#[derive(serde::Serialize)]
struct RegistrationMessageResponse {
    message: String,
    solver_address: Address,
    bond_amount: U256,
    supported_chains: Vec<u64>,
    fee_rate: f64,
}