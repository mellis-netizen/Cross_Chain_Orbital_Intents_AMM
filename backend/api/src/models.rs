use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use redis::aio::MultiplexedConnection;
use ethers::types::{Address, U256, H256};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use metrics_exporter_prometheus::PrometheusHandle;

use crate::config::Config;
use intents_engine::IntentsEngine;

// Application state
#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: PgPool,
    pub redis: MultiplexedConnection,
    pub intents_engine: Arc<IntentsEngine>,
    pub config: Config,
    pub prometheus_handle: PrometheusHandle,
}

// Request/Response models
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitIntentRequest {
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub source_token: Address,
    pub dest_token: Address,
    pub source_amount: U256,
    pub min_dest_amount: U256,
    pub deadline: DateTime<Utc>,
    pub user_address: Address,
    pub signature: String,
    pub nonce: U256,
    pub max_gas_price: Option<U256>,
    pub slippage_tolerance: Option<f64>, // e.g., 0.01 for 1%
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentResponse {
    pub intent_id: H256,
    pub status: String,
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub source_token: Address,
    pub dest_token: Address,
    pub source_amount: U256,
    pub min_dest_amount: U256,
    pub actual_dest_amount: Option<U256>,
    pub deadline: DateTime<Utc>,
    pub user_address: Address,
    pub solver_address: Option<Address>,
    pub execution_tx_hash: Option<H256>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub gas_used: Option<U256>,
    pub fees_paid: Option<U256>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentStatusResponse {
    pub intent_id: H256,
    pub status: String,
    pub progress: IntentProgress,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentProgress {
    pub current_step: String,
    pub steps_completed: u32,
    pub total_steps: u32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolverRegistrationRequest {
    pub solver_address: Address,
    pub bond_amount: U256,
    pub supported_chains: Vec<u64>,
    pub fee_rate: f64, // Basis points, e.g., 30 = 0.3%
    pub contact_info: Option<String>,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolverResponse {
    pub address: Address,
    pub bond_amount: U256,
    pub supported_chains: Vec<u64>,
    pub reputation_score: f64,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_volume: U256,
    pub fee_rate: f64,
    pub is_active: bool,
    pub is_slashed: bool,
    pub last_activity: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub total_intents: u64,
    pub successful_intents: u64,
    pub failed_intents: u64,
    pub pending_intents: u64,
    pub total_volume: U256,
    pub average_execution_time: f64, // seconds
    pub success_rate: f64,
    pub top_chains: Vec<ChainStats>,
    pub top_tokens: Vec<TokenStats>,
    pub solver_performance: Vec<SolverPerformance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainStats {
    pub chain_id: u64,
    pub chain_name: String,
    pub intent_count: u64,
    pub volume: U256,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStats {
    pub token_address: Address,
    pub token_symbol: String,
    pub chain_id: u64,
    pub volume: U256,
    pub transaction_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SolverPerformance {
    pub solver_address: Address,
    pub success_count: u64,
    pub failure_count: u64,
    pub average_execution_time: f64,
    pub reputation_score: f64,
    pub total_volume: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime: u64, // seconds
    pub database: HealthCheck,
    pub redis: HealthCheck,
    pub intent_engine: HealthCheck,
    pub chains: Vec<ChainHealth>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainHealth {
    pub chain_id: u64,
    pub chain_name: String,
    pub status: String,
    pub block_number: Option<u64>,
    pub latency_ms: Option<u64>,
    pub gas_price: Option<U256>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub current_page: u64,
    pub per_page: u64,
    pub total_items: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

// Database models
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct IntentRecord {
    pub id: Uuid,
    pub intent_id: String, // H256 as hex string
    pub source_chain_id: i64,
    pub dest_chain_id: i64,
    pub source_token: String, // Address as hex string
    pub dest_token: String,
    pub source_amount: String, // U256 as decimal string
    pub min_dest_amount: String,
    pub actual_dest_amount: Option<String>,
    pub deadline: DateTime<Utc>,
    pub user_address: String,
    pub solver_address: Option<String>,
    pub status: String,
    pub execution_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub gas_used: Option<String>,
    pub fees_paid: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct SolverRecord {
    pub id: Uuid,
    pub address: String,
    pub bond_amount: String,
    pub supported_chains: Vec<i64>,
    pub reputation_score: f64,
    pub success_count: i64,
    pub failure_count: i64,
    pub total_volume: String,
    pub fee_rate: f64,
    pub is_active: bool,
    pub is_slashed: bool,
    pub last_activity: DateTime<Utc>,
    pub registered_at: DateTime<Utc>,
    pub contact_info: Option<String>,
}

// WebSocket message types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentUpdateMessage {
    pub intent_id: H256,
    pub status: String,
    pub progress: IntentProgress,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataMessage {
    pub chain_id: u64,
    pub token_pair: (Address, Address),
    pub price: f64,
    pub volume_24h: U256,
    pub price_change_24h: f64,
}

// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user address)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
    pub role: String, // User role ("user", "solver", "admin")
}

// Rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub requests_remaining: u32,
    pub reset_time: DateTime<Utc>,
    pub window_size: u32,
}