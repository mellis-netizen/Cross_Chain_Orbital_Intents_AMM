pub mod matcher;
pub mod optimizer;
pub mod executor;
pub mod reputation;
pub mod monitoring;

#[cfg(test)]
mod executor_tests;

#[cfg(test)]
mod tests;

use async_trait::async_trait;
use ethers::types::{Address, U256, H256};
use intents_engine::intent::{Intent, IntentExecution};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolverError {
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[error("Unprofitable intent")]
    Unprofitable,
    
    #[error("Risk limit exceeded")]
    RiskLimitExceeded,
    
    #[error("Chain not supported: {0}")]
    ChainNotSupported(u64),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

pub type Result<T> = std::result::Result<T, SolverError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    pub address: Address,
    pub min_profit_bps: u16,
    pub base_risk_bps: u16,
    pub max_slippage_bps: u16,
    pub supported_chains: Vec<u64>,
    pub oracle_addresses: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SolverMetrics {
    pub total_intents_matched: u64,
    pub total_intents_executed: u64,
    pub total_profit: U256,
    pub average_execution_time: u64,
    pub success_rate: f64,
}

#[async_trait]
pub trait Solver: Send + Sync {
    async fn evaluate_intent(&self, intent: &Intent) -> Result<SolverQuote>;
    async fn match_intent(&self, intent_id: H256, intent: &Intent) -> Result<()>;
    async fn execute_intent(&self, intent_id: H256) -> Result<IntentExecution>;
    fn get_metrics(&self) -> SolverMetrics;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverQuote {
    pub solver: Address,
    pub dest_amount: U256,
    pub profit: U256,
    pub execution_time_estimate: u64,
    pub confidence: f64,
}

pub struct SolverNode {
    config: SolverConfig,
    matcher: Arc<matcher::IntentMatcher>,
    optimizer: Arc<optimizer::RouteOptimizer>,
    executor: Arc<executor::SolverExecutor>,
    reputation: Arc<reputation::ReputationManager>,
}

impl SolverNode {
    pub async fn new(config: SolverConfig) -> Result<Self> {
        let reputation = Arc::new(reputation::ReputationManager::new());
        let matcher = Arc::new(matcher::IntentMatcher::new(reputation.clone()));
        let optimizer = Arc::new(optimizer::RouteOptimizer::new(&config).await?);
        let executor = Arc::new(executor::SolverExecutor::new(config.clone()).await?);

        Ok(Self {
            config,
            matcher,
            optimizer,
            executor,
            reputation,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        // Start monitoring for new intents
        // Start execution loop
        // Start reputation updates
        Ok(())
    }
}

#[async_trait]
impl Solver for SolverNode {
    async fn evaluate_intent(&self, intent: &Intent) -> Result<SolverQuote> {
        // Check if chains are supported
        if !self.config.supported_chains.contains(&intent.source_chain_id) ||
           !self.config.supported_chains.contains(&intent.dest_chain_id) {
            return Err(SolverError::ChainNotSupported(intent.source_chain_id));
        }
        
        // Find optimal route
        let route = self.optimizer.find_best_route(intent).await?;
        
        // Calculate expected output and profit
        let (dest_amount, profit) = self.optimizer.calculate_profit(&route, intent).await?;
        
        // Check profitability
        let profit_bps = profit * U256::from(10000) / intent.source_amount;
        if profit_bps < U256::from(self.config.min_profit_bps) {
            return Err(SolverError::Unprofitable);
        }
        
        // Estimate execution time
        let execution_time = self.estimate_execution_time(&route);
        
        Ok(SolverQuote {
            solver: self.config.address,
            dest_amount,
            profit,
            execution_time_estimate: execution_time,
            confidence: 0.95, // TODO: Calculate based on historical performance
        })
    }
    
    async fn match_intent(&self, intent_id: H256, intent: &Intent) -> Result<()> {
        self.matcher.match_intent(intent_id, intent, &self.config).await
    }
    
    async fn execute_intent(&self, intent_id: H256) -> Result<IntentExecution> {
        self.executor.execute(intent_id).await
    }
    
    fn get_metrics(&self) -> SolverMetrics {
        // TODO: Aggregate metrics from components
        SolverMetrics {
            total_intents_matched: 0,
            total_intents_executed: 0,
            total_profit: U256::zero(),
            average_execution_time: 0,
            success_rate: 0.0,
        }
    }
}

impl SolverNode {
    fn estimate_execution_time(&self, route: &optimizer::Route) -> u64 {
        // Base time for transaction confirmation
        let mut time = 30; // seconds
        
        // Add time for each hop
        time += route.hops.len() as u64 * 15;
        
        // Add buffer for cross-chain messages
        if route.cross_chain {
            time += 60;
        }
        
        time
    }
}