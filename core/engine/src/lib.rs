pub mod intent;
pub mod executor;
pub mod validator;
pub mod state;

use std::sync::Arc;
use tokio::sync::RwLock;
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Invalid intent: {0}")]
    InvalidIntent(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Solver not found: {0}")]
    SolverNotFound(Address),
    
    #[error("Intent expired")]
    IntentExpired,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Chain not supported: {0}")]
    ChainNotSupported(u64),
    
    #[error("Bridge error: {0}")]
    BridgeError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub intents_contract: Address,
    pub orbital_amm_contract: Address,
    pub bridge_contract: Address,
    pub confirmation_blocks: u64,
}

#[derive(Debug, Clone)]
pub struct IntentsEngine {
    chains: Arc<RwLock<Vec<ChainConfig>>>,
    state: Arc<state::EngineState>,
    executor: Arc<executor::IntentExecutor>,
}

impl IntentsEngine {
    pub async fn new(chains: Vec<ChainConfig>) -> Result<Self> {
        let state = Arc::new(state::EngineState::new());
        let executor = Arc::new(executor::IntentExecutor::new(chains.clone(), state.clone()).await?);
        
        Ok(Self {
            chains: Arc::new(RwLock::new(chains)),
            state,
            executor,
        })
    }
    
    pub async fn submit_intent(&self, intent: intent::Intent) -> Result<H256> {
        validator::validate_intent(&intent)?;
        
        let intent_id = self.state.add_intent(intent.clone()).await?;
        
        self.executor.queue_intent(intent_id, intent).await?;
        
        Ok(intent_id)
    }
    
    pub async fn get_intent_status(&self, intent_id: H256) -> Result<intent::IntentStatus> {
        self.state.get_intent_status(intent_id).await
    }
    
    pub async fn add_chain(&self, config: ChainConfig) -> Result<()> {
        let mut chains = self.chains.write().await;
        chains.push(config.clone());
        
        self.executor.add_chain(config).await?;
        
        Ok(())
    }
    
    pub async fn start(&self) -> Result<()> {
        self.executor.start().await
    }
    
    pub async fn stop(&self) -> Result<()> {
        self.executor.stop().await
    }
}