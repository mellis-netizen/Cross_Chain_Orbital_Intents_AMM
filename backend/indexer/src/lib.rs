pub mod indexer;
pub mod events;
pub mod storage;
pub mod config;
pub mod error;
pub mod metrics;

pub use config::IndexerConfig;
pub use error::{IndexerError, Result};
pub use indexer::BlockchainIndexer;

use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Core indexer components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    pub chain_id: u64,
    pub latest_block: u64,
    pub latest_block_hash: H256,
    pub indexed_block: u64,
    pub confirmation_blocks: u64,
    pub is_synced: bool,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedEvent {
    pub id: uuid::Uuid,
    pub chain_id: u64,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub transaction_index: u64,
    pub log_index: u64,
    pub event_type: String,
    pub contract_address: Address,
    pub event_data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub processed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentCreatedEvent {
    pub intent_id: H256,
    pub user: Address,
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub source_token: Address,
    pub dest_token: Address,
    pub source_amount: U256,
    pub min_dest_amount: U256,
    pub deadline: u64,
    pub nonce: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentMatchedEvent {
    pub intent_id: H256,
    pub solver: Address,
    pub execution_price: U256,
    pub estimated_gas: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentExecutedEvent {
    pub intent_id: H256,
    pub solver: Address,
    pub dest_amount: U256,
    pub gas_used: U256,
    pub fees_paid: U256,
    pub execution_hash: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecutedEvent {
    pub pool_address: Address,
    pub user: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out: U256,
    pub fee: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityEvent {
    pub pool_address: Address,
    pub provider: Address,
    pub token0: Address,
    pub token1: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub liquidity: U256,
    pub event_type: LiquidityEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiquidityEventType {
    Add,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeEvent {
    pub bridge_address: Address,
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub token: Address,
    pub amount: U256,
    pub sender: Address,
    pub recipient: Address,
    pub bridge_tx_hash: H256,
    pub event_type: BridgeEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeEventType {
    Deposit,
    Withdrawal,
    Transfer,
}

// Indexer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerStats {
    pub chains: HashMap<u64, ChainStats>,
    pub total_events_indexed: u64,
    pub events_per_second: f64,
    pub total_intents_processed: u64,
    pub uptime: u64,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub chain_id: u64,
    pub blocks_indexed: u64,
    pub events_indexed: u64,
    pub sync_progress: f64,
    pub blocks_behind: u64,
    pub avg_block_time: f64,
    pub last_indexed_block: u64,
}

// Event processing result
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub events_processed: u64,
    pub blocks_processed: u64,
    pub errors: Vec<String>,
    pub processing_time: std::time::Duration,
}

// Indexer configuration per chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainIndexerConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub start_block: u64,
    pub confirmation_blocks: u64,
    pub batch_size: u64,
    pub max_reorg_depth: u64,
    pub contracts: ContractAddresses,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub intents_contract: Address,
    pub orbital_amm_contract: Address,
    pub bridge_contract: Address,
    pub solver_registry: Option<Address>,
}

// Event subscription filters
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub contract_address: Option<Address>,
    pub event_signature: Option<H256>,
    pub topics: Vec<Option<H256>>,
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
}

// Real-time event stream
#[derive(Debug, Clone)]
pub struct EventStream {
    pub chain_id: u64,
    pub receiver: tokio::sync::broadcast::Receiver<IndexedEvent>,
}

// Public API for starting the indexer
pub async fn start_indexer(config: IndexerConfig) -> Result<()> {
    let indexer = BlockchainIndexer::new(config).await?;
    indexer.start().await
}

// Public API for getting indexer stats
pub async fn get_indexer_stats(config: &IndexerConfig) -> Result<IndexerStats> {
    let storage = storage::IndexerStorage::new(&config.database_url).await?;
    storage.get_stats().await
}

// Public API for querying events
pub async fn query_events(
    config: &IndexerConfig,
    filter: EventFilter,
    limit: Option<u64>,
) -> Result<Vec<IndexedEvent>> {
    let storage = storage::IndexerStorage::new(&config.database_url).await?;
    storage.query_events(filter, limit).await
}

// Public API for getting chain state
pub async fn get_chain_state(
    config: &IndexerConfig,
    chain_id: u64,
) -> Result<Option<ChainState>> {
    let storage = storage::IndexerStorage::new(&config.database_url).await?;
    storage.get_chain_state(chain_id).await
}