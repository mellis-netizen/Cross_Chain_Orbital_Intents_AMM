# Virtual Pool State Management

## Overview
The Virtual Pool State Management system enables liquidity aggregation across multiple chains and protocols by maintaining a unified virtual state that represents distributed liquidity pools. This allows the Orbital AMM to provide deep liquidity without requiring all assets to be on a single chain.

## Architecture

### Core Components

#### 1. VirtualPoolState
Maintains the aggregate state of liquidity across multiple physical pools.

```rust
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPoolState {
    /// Unique pool identifier
    pub pool_id: H256,

    /// Token pair
    pub token0: Address,
    pub token1: Address,

    /// Aggregate virtual reserves
    pub virtual_reserve0: U256,
    pub virtual_reserve1: U256,

    /// Physical pools backing this virtual pool
    pub physical_pools: Vec<PhysicalPool>,

    /// Last update timestamp
    pub last_update: u64,

    /// Total liquidity locked value (USD)
    pub tvl: U256,

    /// Pool utilization ratio (0-10000 basis points)
    pub utilization: u16,

    /// State merkle root for verification
    pub state_root: H256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalPool {
    /// Chain ID where pool exists
    pub chain_id: u64,

    /// Pool contract address
    pub pool_address: Address,

    /// Protocol type (UniswapV3, Curve, etc.)
    pub protocol: PoolProtocol,

    /// Real reserves in this pool
    pub reserve0: U256,
    pub reserve1: U256,

    /// Pool's contribution weight to virtual pool (basis points)
    pub weight: u16,

    /// Fee tier (basis points)
    pub fee: u16,

    /// Last synchronized block
    pub last_sync_block: u64,

    /// Pool health status
    pub status: PoolStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolProtocol {
    OrbitalAMM,
    UniswapV2,
    UniswapV3,
    Curve,
    Balancer,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolStatus {
    Active,
    Degraded,
    Paused,
    Deprecated,
}
```

#### 2. StateManager
Manages virtual pool state updates and synchronization.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StateError {
    #[error("Pool not found: {0}")]
    PoolNotFound(H256),

    #[error("Synchronization failed: {0}")]
    SyncFailed(String),

    #[error("State verification failed: {0}")]
    VerificationFailed(String),

    #[error("Insufficient liquidity in pool {0}")]
    InsufficientLiquidity(H256),

    #[error("Chain not available: {0}")]
    ChainUnavailable(u64),
}

pub type StateResult<T> = std::result::Result<T, StateError>;

pub struct VirtualPoolManager {
    /// Virtual pool states
    pools: Arc<RwLock<HashMap<H256, VirtualPoolState>>>,

    /// Chain synchronizers
    chain_syncs: Arc<RwLock<HashMap<u64, ChainSynchronizer>>>,

    /// State verifier
    verifier: Arc<StateVerifier>,

    /// Rebalancing strategy
    rebalancer: Arc<PoolRebalancer>,
}

impl VirtualPoolManager {
    pub async fn new() -> StateResult<Self> {
        Ok(Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            chain_syncs: Arc::new(RwLock::new(HashMap::new())),
            verifier: Arc::new(StateVerifier::new()),
            rebalancer: Arc::new(PoolRebalancer::new()),
        })
    }

    /// Create a new virtual pool
    pub async fn create_virtual_pool(
        &self,
        token0: Address,
        token1: Address,
        physical_pools: Vec<PhysicalPool>,
    ) -> StateResult<H256> {
        let pool_id = self.compute_pool_id(token0, token1);

        // Verify all physical pools
        for pool in &physical_pools {
            self.verify_physical_pool(pool).await?;
        }

        // Calculate initial virtual reserves
        let (virtual_reserve0, virtual_reserve1) =
            self.calculate_virtual_reserves(&physical_pools);

        let state = VirtualPoolState {
            pool_id,
            token0,
            token1,
            virtual_reserve0,
            virtual_reserve1,
            physical_pools,
            last_update: Self::current_timestamp(),
            tvl: U256::zero(), // Calculate separately
            utilization: 0,
            state_root: H256::zero(), // Calculate merkle root
        };

        let mut pools = self.pools.write().await;
        pools.insert(pool_id, state);

        Ok(pool_id)
    }

    /// Synchronize virtual pool state with physical pools
    pub async fn sync_pool(&self, pool_id: H256) -> StateResult<()> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(&pool_id)
            .ok_or(StateError::PoolNotFound(pool_id))?;

        // Sync each physical pool
        for physical_pool in &mut pool.physical_pools {
            self.sync_physical_pool(physical_pool).await?;
        }

        // Recalculate virtual reserves
        let (new_reserve0, new_reserve1) =
            self.calculate_virtual_reserves(&pool.physical_pools);

        pool.virtual_reserve0 = new_reserve0;
        pool.virtual_reserve1 = new_reserve1;
        pool.last_update = Self::current_timestamp();

        // Update state root
        pool.state_root = self.verifier.compute_state_root(pool);

        Ok(())
    }

    /// Get virtual pool quote for swap
    pub async fn get_virtual_quote(
        &self,
        pool_id: H256,
        token_in: Address,
        amount_in: U256,
    ) -> StateResult<U256> {
        let pools = self.pools.read().await;
        let pool = pools.get(&pool_id)
            .ok_or(StateError::PoolNotFound(pool_id))?;

        let (reserve_in, reserve_out) = if pool.token0 == token_in {
            (pool.virtual_reserve0, pool.virtual_reserve1)
        } else {
            (pool.virtual_reserve1, pool.virtual_reserve0)
        };

        // Constant product formula with virtual reserves
        let amount_out = Self::calculate_output(
            amount_in,
            reserve_in,
            reserve_out,
            30, // 0.3% fee
        );

        Ok(amount_out)
    }

    /// Execute swap across physical pools
    pub async fn execute_virtual_swap(
        &self,
        pool_id: H256,
        token_in: Address,
        amount_in: U256,
        min_amount_out: U256,
    ) -> StateResult<SwapExecution> {
        // Get pool state
        let pools = self.pools.read().await;
        let pool = pools.get(&pool_id)
            .ok_or(StateError::PoolNotFound(pool_id))?
            .clone();
        drop(pools);

        // Calculate optimal distribution across physical pools
        let distribution = self.rebalancer.calculate_swap_distribution(
            &pool,
            token_in,
            amount_in,
        ).await?;

        // Execute swaps on each physical pool
        let mut executions = Vec::new();
        for (physical_pool, swap_amount) in distribution {
            let execution = self.execute_physical_swap(
                &physical_pool,
                token_in,
                swap_amount,
            ).await?;
            executions.push(execution);
        }

        // Aggregate results
        let total_output: U256 = executions.iter()
            .map(|e| e.amount_out)
            .sum();

        if total_output < min_amount_out {
            return Err(StateError::InsufficientLiquidity(pool_id));
        }

        // Update virtual pool state
        self.update_after_swap(pool_id, token_in, amount_in, total_output).await?;

        Ok(SwapExecution {
            pool_id,
            amount_in,
            amount_out: total_output,
            physical_executions: executions,
            gas_used: U256::zero(), // Calculate actual gas
        })
    }

    // Helper methods

    fn calculate_virtual_reserves(&self, pools: &[PhysicalPool]) -> (U256, U256) {
        let mut total_reserve0 = U256::zero();
        let mut total_reserve1 = U256::zero();

        for pool in pools {
            if pool.status == PoolStatus::Active {
                // Weight reserves by pool weight
                let weight_multiplier = U256::from(pool.weight);
                total_reserve0 += pool.reserve0 * weight_multiplier / U256::from(10000);
                total_reserve1 += pool.reserve1 * weight_multiplier / U256::from(10000);
            }
        }

        (total_reserve0, total_reserve1)
    }

    fn calculate_output(
        amount_in: U256,
        reserve_in: U256,
        reserve_out: U256,
        fee_bps: u16,
    ) -> U256 {
        let amount_in_with_fee = amount_in * U256::from(10000 - fee_bps) / U256::from(10000);
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in + amount_in_with_fee;
        numerator / denominator
    }

    async fn verify_physical_pool(&self, pool: &PhysicalPool) -> StateResult<()> {
        // Verify pool exists and is accessible
        let syncs = self.chain_syncs.read().await;
        let sync = syncs.get(&pool.chain_id)
            .ok_or(StateError::ChainUnavailable(pool.chain_id))?;

        sync.verify_pool(pool).await
    }

    async fn sync_physical_pool(&self, pool: &mut PhysicalPool) -> StateResult<()> {
        let syncs = self.chain_syncs.read().await;
        let sync = syncs.get(&pool.chain_id)
            .ok_or(StateError::ChainUnavailable(pool.chain_id))?;

        let (reserve0, reserve1, block) = sync.fetch_pool_state(pool.pool_address).await?;

        pool.reserve0 = reserve0;
        pool.reserve1 = reserve1;
        pool.last_sync_block = block;

        Ok(())
    }

    async fn execute_physical_swap(
        &self,
        pool: &PhysicalPool,
        token_in: Address,
        amount_in: U256,
    ) -> StateResult<PhysicalSwapExecution> {
        let syncs = self.chain_syncs.read().await;
        let sync = syncs.get(&pool.chain_id)
            .ok_or(StateError::ChainUnavailable(pool.chain_id))?;

        sync.execute_swap(pool, token_in, amount_in).await
    }

    async fn update_after_swap(
        &self,
        pool_id: H256,
        token_in: Address,
        amount_in: U256,
        amount_out: U256,
    ) -> StateResult<()> {
        let mut pools = self.pools.write().await;
        let pool = pools.get_mut(&pool_id)
            .ok_or(StateError::PoolNotFound(pool_id))?;

        // Update virtual reserves
        if pool.token0 == token_in {
            pool.virtual_reserve0 += amount_in;
            pool.virtual_reserve1 = pool.virtual_reserve1.saturating_sub(amount_out);
        } else {
            pool.virtual_reserve1 += amount_in;
            pool.virtual_reserve0 = pool.virtual_reserve0.saturating_sub(amount_out);
        }

        pool.last_update = Self::current_timestamp();

        Ok(())
    }

    fn compute_pool_id(&self, token0: Address, token1: Address) -> H256 {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token0.as_bytes());
        hasher.update(token1.as_bytes());
        H256::from_slice(&hasher.finalize())
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[derive(Debug, Clone)]
pub struct SwapExecution {
    pub pool_id: H256,
    pub amount_in: U256,
    pub amount_out: U256,
    pub physical_executions: Vec<PhysicalSwapExecution>,
    pub gas_used: U256,
}

#[derive(Debug, Clone)]
pub struct PhysicalSwapExecution {
    pub chain_id: u64,
    pub pool_address: Address,
    pub amount_in: U256,
    pub amount_out: U256,
    pub tx_hash: H256,
}
```

#### 3. ChainSynchronizer
Handles synchronization with individual blockchain networks.

```rust
use ethers::providers::{Provider, Http};
use ethers::contract::Contract;

pub struct ChainSynchronizer {
    chain_id: u64,
    provider: Provider<Http>,
    contracts: HashMap<Address, Contract<Provider<Http>>>,
}

impl ChainSynchronizer {
    pub async fn new(chain_id: u64, rpc_url: String) -> StateResult<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| StateError::ChainUnavailable(chain_id))?;

        Ok(Self {
            chain_id,
            provider,
            contracts: HashMap::new(),
        })
    }

    pub async fn verify_pool(&self, pool: &PhysicalPool) -> StateResult<()> {
        // Check pool contract exists and has expected interface
        let code = self.provider
            .get_code(pool.pool_address, None)
            .await
            .map_err(|e| StateError::SyncFailed(e.to_string()))?;

        if code.is_empty() {
            return Err(StateError::SyncFailed("Pool contract not found".to_string()));
        }

        Ok(())
    }

    pub async fn fetch_pool_state(
        &self,
        pool_address: Address,
    ) -> StateResult<(U256, U256, u64)> {
        // Fetch reserves and current block
        // Implementation depends on pool protocol
        // This is a simplified example

        let block_number = self.provider
            .get_block_number()
            .await
            .map_err(|e| StateError::SyncFailed(e.to_string()))?;

        // Mock data - in production, call actual pool contract
        Ok((U256::from(1000000), U256::from(2000000), block_number.as_u64()))
    }

    pub async fn execute_swap(
        &self,
        pool: &PhysicalPool,
        token_in: Address,
        amount_in: U256,
    ) -> StateResult<PhysicalSwapExecution> {
        // Execute actual swap on-chain
        // Return execution details

        Ok(PhysicalSwapExecution {
            chain_id: self.chain_id,
            pool_address: pool.pool_address,
            amount_in,
            amount_out: U256::zero(), // Get from swap result
            tx_hash: H256::zero(), // Get from transaction
        })
    }
}
```

#### 4. StateVerifier
Verifies virtual pool state integrity.

```rust
use sha2::{Sha256, Digest};

pub struct StateVerifier;

impl StateVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_state_root(&self, pool: &VirtualPoolState) -> H256 {
        let mut hasher = Sha256::new();

        // Hash pool data
        hasher.update(pool.pool_id.as_bytes());
        hasher.update(pool.token0.as_bytes());
        hasher.update(pool.token1.as_bytes());
        hasher.update(pool.virtual_reserve0.to_string().as_bytes());
        hasher.update(pool.virtual_reserve1.to_string().as_bytes());
        hasher.update(&pool.last_update.to_le_bytes());

        // Hash physical pools
        for physical in &pool.physical_pools {
            hasher.update(&physical.chain_id.to_le_bytes());
            hasher.update(physical.pool_address.as_bytes());
            hasher.update(physical.reserve0.to_string().as_bytes());
            hasher.update(physical.reserve1.to_string().as_bytes());
        }

        H256::from_slice(&hasher.finalize())
    }

    pub fn verify_state(
        &self,
        pool: &VirtualPoolState,
        claimed_root: H256,
    ) -> bool {
        let computed_root = self.compute_state_root(pool);
        computed_root == claimed_root
    }
}
```

#### 5. PoolRebalancer
Optimizes liquidity distribution across physical pools.

```rust
pub struct PoolRebalancer;

impl PoolRebalancer {
    pub fn new() -> Self {
        Self
    }

    pub async fn calculate_swap_distribution(
        &self,
        pool: &VirtualPoolState,
        token_in: Address,
        amount_in: U256,
    ) -> StateResult<Vec<(PhysicalPool, U256)>> {
        let mut distribution = Vec::new();
        let mut remaining = amount_in;

        // Sort pools by best price
        let mut pools: Vec<_> = pool.physical_pools.iter()
            .filter(|p| p.status == PoolStatus::Active)
            .collect();

        pools.sort_by(|a, b| {
            let price_a = self.calculate_price(a, token_in);
            let price_b = self.calculate_price(b, token_in);
            price_b.cmp(&price_a) // Higher price first
        });

        // Distribute amount across pools
        for pool in pools {
            if remaining == U256::zero() {
                break;
            }

            // Calculate max amount this pool can handle efficiently
            let max_amount = self.calculate_max_efficient_amount(pool, token_in);
            let swap_amount = remaining.min(max_amount);

            distribution.push((pool.clone(), swap_amount));
            remaining -= swap_amount;
        }

        if remaining > U256::zero() {
            return Err(StateError::InsufficientLiquidity(pool.pool_id));
        }

        Ok(distribution)
    }

    fn calculate_price(&self, pool: &PhysicalPool, token_in: Address) -> U256 {
        if pool.token0 == token_in {
            pool.reserve1 * U256::from(1e18) / pool.reserve0
        } else {
            pool.reserve0 * U256::from(1e18) / pool.reserve1
        }
    }

    fn calculate_max_efficient_amount(
        &self,
        pool: &PhysicalPool,
        token_in: Address,
    ) -> U256 {
        let reserve_in = if pool.token0 == token_in {
            pool.reserve0
        } else {
            pool.reserve1
        };

        // Max 5% of reserve to avoid large slippage
        reserve_in * U256::from(5) / U256::from(100)
    }
}
```

## State Transitions

### Pool Creation
1. Validate physical pools exist on their respective chains
2. Fetch initial reserves from all physical pools
3. Calculate weighted virtual reserves
4. Compute initial state root
5. Register virtual pool in state manager

### Swap Execution
1. Calculate optimal distribution across physical pools
2. Execute parallel swaps on respective chains
3. Aggregate results
4. Update virtual pool reserves
5. Compute new state root
6. Emit state update events

### Synchronization
1. Periodic sync of physical pool reserves
2. Update virtual reserves based on weighted aggregation
3. Recompute state root
4. Detect and handle pool degradation

## Security Considerations

### 1. State Consistency
- Atomic updates across physical pools
- Rollback mechanism for failed multi-pool swaps
- State root verification for all updates

### 2. Access Control
- Only authorized solvers can execute swaps
- Pool admin can pause/unpause physical pools
- Emergency shutdown mechanism

### 3. Oracle Manipulation
- Time-weighted average prices (TWAP) for pool valuation
- Multiple price sources for verification
- Abnormal reserve change detection

### 4. Front-running Protection
- Commit-reveal for large swaps
- MEV-resistant execution ordering
- Slippage tolerance enforcement

## Performance Optimizations

### 1. Caching
- Cache frequently accessed pool states
- Batch reserve updates
- Lazy state root computation

### 2. Parallel Execution
- Concurrent physical pool queries
- Parallel swap execution across chains
- Async state synchronization

### 3. Gas Optimization
- Batch cross-chain messages
- Minimize on-chain storage
- Efficient state encoding

## Testing Strategy

### Unit Tests
- Virtual reserve calculations
- State root computation
- Swap distribution algorithms

### Integration Tests
- Multi-chain synchronization
- Cross-chain swap execution
- State recovery after failures

### Stress Tests
- High-frequency trading simulation
- Large swap handling
- Network partition scenarios
