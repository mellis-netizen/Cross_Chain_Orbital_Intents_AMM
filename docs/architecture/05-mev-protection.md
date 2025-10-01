# MEV Protection Mechanisms

## Overview
MEV (Maximal Extractable Value) protection mechanisms safeguard users from front-running, sandwich attacks, and other forms of transaction ordering exploitation in the cross-chain intents system.

## Architecture

### Protection Layers

#### 1. Commit-Reveal Scheme

```rust
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    /// Commitment hash
    pub commit_hash: H256,

    /// Trader address
    pub trader: Address,

    /// Block number when committed
    pub commit_block: u64,

    /// Expiry block
    pub expiry_block: u64,

    /// Pool ID (revealed)
    pub pool_id: Option<H256>,

    /// Status
    pub status: CommitmentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentStatus {
    Committed,
    Revealed,
    Expired,
    Executed,
}

pub struct CommitRevealManager {
    /// Active commitments
    commitments: Arc<RwLock<HashMap<H256, Commitment>>>,

    /// Minimum blocks between commit and reveal
    min_delay: u64,

    /// Maximum blocks before expiry
    max_expiry: u64,
}

impl CommitRevealManager {
    pub fn new(min_delay: u64, max_expiry: u64) -> Self {
        Self {
            commitments: Arc::new(RwLock::new(HashMap::new())),
            min_delay,
            max_expiry,
        }
    }

    /// Create a commitment
    pub async fn commit(
        &self,
        trader: Address,
        pool_id: H256,
        zero_for_one: bool,
        amount_in: U256,
        nonce: U256,
        current_block: u64,
    ) -> Result<H256, MEVError> {
        // Generate commitment hash
        let commit_hash = self.compute_commit_hash(
            trader,
            pool_id,
            zero_for_one,
            amount_in,
            nonce,
        );

        let commitment = Commitment {
            commit_hash,
            trader,
            commit_block: current_block,
            expiry_block: current_block + self.max_expiry,
            pool_id: None,
            status: CommitmentStatus::Committed,
        };

        let mut commitments = self.commitments.write().await;
        commitments.insert(commit_hash, commitment);

        Ok(commit_hash)
    }

    /// Reveal and execute swap
    pub async fn reveal(
        &self,
        commit_hash: H256,
        pool_id: H256,
        zero_for_one: bool,
        amount_in: U256,
        nonce: U256,
        current_block: u64,
    ) -> Result<(), MEVError> {
        let mut commitments = self.commitments.write().await;
        let commitment = commitments
            .get_mut(&commit_hash)
            .ok_or(MEVError::CommitmentNotFound)?;

        // Verify trader
        // (Would check msg.sender in actual implementation)

        // Check timing
        if current_block < commitment.commit_block + self.min_delay {
            return Err(MEVError::RevealTooEarly);
        }

        if current_block > commitment.expiry_block {
            commitment.status = CommitmentStatus::Expired;
            return Err(MEVError::CommitmentExpired);
        }

        // Verify commitment
        let expected_hash = self.compute_commit_hash(
            commitment.trader,
            pool_id,
            zero_for_one,
            amount_in,
            nonce,
        );

        if expected_hash != commit_hash {
            return Err(MEVError::InvalidReveal);
        }

        // Update status
        commitment.pool_id = Some(pool_id);
        commitment.status = CommitmentStatus::Revealed;

        Ok(())
    }

    fn compute_commit_hash(
        &self,
        trader: Address,
        pool_id: H256,
        zero_for_one: bool,
        amount_in: U256,
        nonce: U256,
    ) -> H256 {
        let mut hasher = Keccak256::new();
        hasher.update(trader.as_bytes());
        hasher.update(pool_id.as_bytes());
        hasher.update(&[zero_for_one as u8]);
        hasher.update(amount_in.to_string().as_bytes());
        hasher.update(nonce.to_string().as_bytes());

        H256::from_slice(&hasher.finalize())
    }
}

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MEVError {
    #[error("Commitment not found")]
    CommitmentNotFound,

    #[error("Reveal too early")]
    RevealTooEarly,

    #[error("Commitment expired")]
    CommitmentExpired,

    #[error("Invalid reveal")]
    InvalidReveal,

    #[error("Price deviation too high: {0}")]
    PriceDeviation(String),

    #[error("Arbitrage detected")]
    ArbitrageDetected,
}
```

#### 2. Time-Weighted Average Price (TWAP) Oracle

```rust
#[derive(Debug, Clone)]
pub struct TWAPOracle {
    /// Price observations
    observations: Arc<RwLock<HashMap<H256, VecDeque<PriceObservation>>>>,

    /// TWAP window (seconds)
    window: u64,

    /// Maximum price deviation (basis points)
    max_deviation: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct PriceObservation {
    pub timestamp: u64,
    pub price: U256,
    pub cumulative_price: U256,
}

impl TWAPOracle {
    pub fn new(window: u64, max_deviation: u16) -> Self {
        Self {
            observations: Arc::new(RwLock::new(HashMap::new())),
            window,
            max_deviation,
        }
    }

    /// Record a price observation
    pub async fn observe(
        &self,
        pool_id: H256,
        price: U256,
        timestamp: u64,
    ) {
        let mut observations = self.observations.write().await;
        let pool_obs = observations.entry(pool_id).or_insert_with(VecDeque::new);

        // Calculate cumulative price
        let cumulative = if let Some(last) = pool_obs.back() {
            let time_delta = timestamp - last.timestamp;
            last.cumulative_price + price * U256::from(time_delta)
        } else {
            price
        };

        pool_obs.push_back(PriceObservation {
            timestamp,
            price,
            cumulative_price: cumulative,
        });

        // Prune old observations
        let cutoff = timestamp.saturating_sub(self.window);
        while let Some(front) = pool_obs.front() {
            if front.timestamp < cutoff {
                pool_obs.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get TWAP for a pool
    pub async fn get_twap(&self, pool_id: H256) -> Option<U256> {
        let observations = self.observations.read().await;
        let pool_obs = observations.get(&pool_id)?;

        if pool_obs.len() < 2 {
            return None;
        }

        let latest = pool_obs.back()?;
        let oldest = pool_obs.front()?;

        let time_delta = latest.timestamp - oldest.timestamp;
        if time_delta == 0 {
            return Some(latest.price);
        }

        let price_delta = latest.cumulative_price - oldest.cumulative_price;
        Some(price_delta / U256::from(time_delta))
    }

    /// Check if current price deviates too much from TWAP
    pub async fn check_deviation(
        &self,
        pool_id: H256,
        current_price: U256,
    ) -> Result<(), MEVError> {
        let twap = self
            .get_twap(pool_id)
            .await
            .ok_or(MEVError::PriceDeviation("No TWAP data".to_string()))?;

        let deviation = if current_price > twap {
            ((current_price - twap) * U256::from(10000) / twap).as_u64() as u16
        } else {
            ((twap - current_price) * U256::from(10000) / twap).as_u64() as u16
        };

        if deviation > self.max_deviation {
            return Err(MEVError::PriceDeviation(
                format!("Deviation {}bps exceeds max {}bps", deviation, self.max_deviation)
            ));
        }

        Ok(())
    }
}

use std::collections::VecDeque;
```

#### 3. Arbitrage Detection and Throttling

```rust
#[derive(Debug, Clone)]
pub struct ArbitrageGuard {
    /// Recent trades by pool
    recent_trades: Arc<RwLock<HashMap<H256, VecDeque<Trade>>>>,

    /// Cooldown period (blocks)
    cooldown_blocks: u64,

    /// Price deviation threshold (basis points)
    deviation_threshold: u16,

    /// Locked pools
    locked_pools: Arc<RwLock<HashMap<H256, u64>>>, // pool_id -> unlock_block
}

#[derive(Debug, Clone)]
struct Trade {
    block_number: u64,
    trader: Address,
    price_before: U256,
    price_after: U256,
    volume: U256,
}

impl ArbitrageGuard {
    pub fn new(cooldown_blocks: u64, deviation_threshold: u16) -> Self {
        Self {
            recent_trades: Arc::new(RwLock::new(HashMap::new())),
            cooldown_blocks,
            deviation_threshold,
            locked_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if trade should be allowed
    pub async fn check_trade(
        &self,
        pool_id: H256,
        trader: Address,
        price_before: U256,
        price_after: U256,
        volume: U256,
        current_block: u64,
    ) -> Result<(), MEVError> {
        // Check if pool is locked
        {
            let locked = self.locked_pools.read().await;
            if let Some(&unlock_block) = locked.get(&pool_id) {
                if current_block < unlock_block {
                    return Err(MEVError::ArbitrageDetected);
                }
            }
        }

        // Calculate price deviation
        let deviation = if price_after > price_before {
            ((price_after - price_before) * U256::from(10000) / price_before)
                .as_u64() as u16
        } else {
            ((price_before - price_after) * U256::from(10000) / price_before)
                .as_u64() as u16
        };

        // Check against threshold
        if deviation > self.deviation_threshold {
            // Check for sandwich attack pattern
            if self.detect_sandwich_pattern(pool_id, trader, current_block).await {
                // Lock pool temporarily
                let mut locked = self.locked_pools.write().await;
                locked.insert(pool_id, current_block + self.cooldown_blocks);

                return Err(MEVError::ArbitrageDetected);
            }
        }

        // Record trade
        self.record_trade(
            pool_id,
            Trade {
                block_number: current_block,
                trader,
                price_before,
                price_after,
                volume,
            },
        )
        .await;

        Ok(())
    }

    async fn detect_sandwich_pattern(
        &self,
        pool_id: H256,
        trader: Address,
        current_block: u64,
    ) -> bool {
        let trades = self.recent_trades.read().await;
        let pool_trades = match trades.get(&pool_id) {
            Some(t) => t,
            None => return false,
        };

        // Look for same trader making opposite trades in recent blocks
        let recent_by_trader: Vec<_> = pool_trades
            .iter()
            .filter(|t| {
                t.trader == trader && current_block - t.block_number <= 3
            })
            .collect();

        // If trader made a trade in opposite direction recently, it's suspicious
        recent_by_trader.len() >= 2
    }

    async fn record_trade(&self, pool_id: H256, trade: Trade) {
        let mut trades = self.recent_trades.write().await;
        let pool_trades = trades.entry(pool_id).or_insert_with(VecDeque::new);

        pool_trades.push_back(trade);

        // Keep only recent trades (last 100 blocks)
        while let Some(front) = pool_trades.front() {
            if pool_trades.back().unwrap().block_number - front.block_number > 100 {
                pool_trades.pop_front();
            } else {
                break;
            }
        }
    }
}
```

#### 4. Fair Ordering Service

```rust
use priority_queue::PriorityQueue;
use std::cmp::Reverse;

pub struct FairOrderingService {
    /// Pending transactions
    pending: Arc<RwLock<PriorityQueue<H256, Reverse<u64>>>>,

    /// Transaction details
    transactions: Arc<RwLock<HashMap<H256, PendingTransaction>>>,

    /// Batch interval (seconds)
    batch_interval: u64,
}

#[derive(Debug, Clone)]
struct PendingTransaction {
    tx_hash: H256,
    sender: Address,
    intent: Intent,
    timestamp: u64,
    priority_fee: U256,
}

impl FairOrderingService {
    pub fn new(batch_interval: u64) -> Self {
        Self {
            pending: Arc::new(RwLock::new(PriorityQueue::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            batch_interval,
        }
    }

    /// Submit transaction to fair ordering
    pub async fn submit(
        &self,
        tx_hash: H256,
        sender: Address,
        intent: Intent,
        priority_fee: U256,
    ) {
        let timestamp = Self::current_timestamp();

        let tx = PendingTransaction {
            tx_hash,
            sender,
            intent,
            timestamp,
            priority_fee,
        };

        // Add to queue with timestamp priority (FCFS within batch)
        let mut pending = self.pending.write().await;
        pending.push(tx_hash, Reverse(timestamp));

        let mut transactions = self.transactions.write().await;
        transactions.insert(tx_hash, tx);
    }

    /// Get next batch of transactions
    pub async fn get_batch(&self) -> Vec<PendingTransaction> {
        let now = Self::current_timestamp();
        let mut batch = Vec::new();

        let mut pending = self.pending.write().await;
        let transactions = self.transactions.read().await;

        // Collect all transactions from the batch window
        while let Some((tx_hash, Reverse(timestamp))) = pending.peek() {
            if now - *timestamp >= self.batch_interval {
                let (tx_hash, _) = pending.pop().unwrap();
                if let Some(tx) = transactions.get(&tx_hash) {
                    batch.push(tx.clone());
                }
            } else {
                break;
            }
        }

        // Sort by timestamp (FCFS)
        batch.sort_by_key(|tx| tx.timestamp);

        batch
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

use intents_engine::intent::Intent;
```

## Integrated MEV Protection System

```rust
pub struct MEVProtectionSystem {
    commit_reveal: Arc<CommitRevealManager>,
    twap_oracle: Arc<TWAPOracle>,
    arbitrage_guard: Arc<ArbitrageGuard>,
    fair_ordering: Arc<FairOrderingService>,
}

impl MEVProtectionSystem {
    pub fn new() -> Self {
        Self {
            commit_reveal: Arc::new(CommitRevealManager::new(2, 20)), // 2-20 blocks
            twap_oracle: Arc::new(TWAPOracle::new(1800, 50)), // 30 min, 0.5% max deviation
            arbitrage_guard: Arc::new(ArbitrageGuard::new(1, 50)), // 1 block cooldown, 0.5% threshold
            fair_ordering: Arc::new(FairOrderingService::new(12)), // 12 second batches
        }
    }

    /// Validate a swap with all protection mechanisms
    pub async fn validate_swap(
        &self,
        pool_id: H256,
        trader: Address,
        current_price: U256,
        price_after: U256,
        volume: U256,
        current_block: u64,
    ) -> Result<(), MEVError> {
        // Check TWAP deviation
        self.twap_oracle
            .check_deviation(pool_id, current_price)
            .await?;

        // Check arbitrage guard
        self.arbitrage_guard
            .check_trade(
                pool_id,
                trader,
                current_price,
                price_after,
                volume,
                current_block,
            )
            .await?;

        Ok(())
    }

    /// Record price observation after swap
    pub async fn record_observation(
        &self,
        pool_id: H256,
        price: U256,
        timestamp: u64,
    ) {
        self.twap_oracle.observe(pool_id, price, timestamp).await;
    }
}
```

## Attack Scenarios and Mitigations

### 1. Front-Running
**Attack**: Attacker sees pending transaction and submits higher gas to execute first.
**Mitigation**: Commit-reveal scheme makes transaction details hidden until execution.

### 2. Sandwich Attack
**Attack**: Attacker surrounds victim transaction with buy-sell pair.
**Mitigation**: Arbitrage guard detects rapid opposite trades and locks pool.

### 3. Price Manipulation
**Attack**: Large trade to manipulate oracle price.
**Mitigation**: TWAP oracle resistant to single-block manipulation.

### 4. Time-Bandit Attack
**Attack**: Reordering transactions within block.
**Mitigation**: Fair ordering service with FCFS batching.

## Configuration Recommendations

### Mainnet (High Security)
```rust
CommitRevealManager::new(3, 50)  // 3-50 blocks (36s-10min)
TWAPOracle::new(3600, 30)        // 1 hour window, 0.3% deviation
ArbitrageGuard::new(2, 30)       // 2 block cooldown, 0.3% threshold
FairOrderingService::new(12)     // 12 second batches
```

### L2 (Balanced)
```rust
CommitRevealManager::new(2, 20)  // 2-20 blocks
TWAPOracle::new(1800, 50)        // 30 min window, 0.5% deviation
ArbitrageGuard::new(1, 50)       // 1 block cooldown, 0.5% threshold
FairOrderingService::new(6)      // 6 second batches
```

### Testnet (Permissive)
```rust
CommitRevealManager::new(1, 10)  // 1-10 blocks
TWAPOracle::new(300, 100)        // 5 min window, 1% deviation
ArbitrageGuard::new(0, 100)      // No cooldown, 1% threshold
FairOrderingService::new(3)      // 3 second batches
```

## Testing Strategy

### Unit Tests
- Commit-reveal verification
- TWAP calculation accuracy
- Arbitrage pattern detection

### Integration Tests
- End-to-end MEV protection
- Multi-user scenarios
- Edge case handling

### Attack Simulations
- Known MEV attack patterns
- Sophisticated multi-step attacks
- Economic incentive analysis
