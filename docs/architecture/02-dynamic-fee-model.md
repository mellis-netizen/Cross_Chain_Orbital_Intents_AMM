# Dynamic Fee Model

## Overview
The Dynamic Fee Model automatically adjusts trading fees based on market conditions, pool utilization, and volatility to optimize for liquidity provision returns while maintaining competitive pricing for traders.

## Architecture

### Core Components

#### 1. FeeCalculator
Computes dynamic fees based on multiple market factors.

```rust
use ethers::types::{U256, Address};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeConfig {
    /// Base fee rate (basis points)
    pub base_fee: u16,

    /// Minimum fee rate (basis points)
    pub min_fee: u16,

    /// Maximum fee rate (basis points)
    pub max_fee: u16,

    /// Volatility weight factor (0-10000)
    pub volatility_weight: u16,

    /// Volume weight factor (0-10000)
    pub volume_weight: u16,

    /// Utilization weight factor (0-10000)
    pub utilization_weight: u16,

    /// Time window for volatility calculation (seconds)
    pub volatility_window: u64,

    /// Time window for volume calculation (seconds)
    pub volume_window: u64,
}

impl Default for FeeConfig {
    fn default() -> Self {
        Self {
            base_fee: 30,           // 0.3%
            min_fee: 5,             // 0.05%
            max_fee: 100,           // 1%
            volatility_weight: 4000, // 40%
            volume_weight: 3000,    // 30%
            utilization_weight: 3000, // 30%
            volatility_window: 1800, // 30 minutes
            volume_window: 86400,   // 24 hours
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeState {
    /// Current dynamic fee (basis points)
    pub current_fee: u16,

    /// Historical volatility
    pub volatility: f64,

    /// 24h trading volume
    pub volume_24h: U256,

    /// Pool utilization ratio (0-1)
    pub utilization: f64,

    /// Last update timestamp
    pub last_update: u64,

    /// Fee history for analysis
    pub fee_history: VecDeque<FeeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSnapshot {
    pub timestamp: u64,
    pub fee: u16,
    pub volatility: f64,
    pub volume: U256,
    pub utilization: f64,
}

pub struct FeeCalculator {
    config: FeeConfig,
}

impl FeeCalculator {
    pub fn new(config: FeeConfig) -> Self {
        Self { config }
    }

    /// Calculate dynamic fee based on current market conditions
    pub fn calculate_fee(
        &self,
        pool_state: &PoolState,
        price_history: &[PricePoint],
        volume_history: &[VolumePoint],
    ) -> u16 {
        let volatility_factor = self.calculate_volatility_factor(price_history);
        let volume_factor = self.calculate_volume_factor(volume_history);
        let utilization_factor = self.calculate_utilization_factor(pool_state);

        // Weighted average of factors
        let total_weight = self.config.volatility_weight
            + self.config.volume_weight
            + self.config.utilization_weight;

        let weighted_adjustment = (
            volatility_factor * self.config.volatility_weight as f64
            + volume_factor * self.config.volume_weight as f64
            + utilization_factor * self.config.utilization_weight as f64
        ) / total_weight as f64;

        // Apply adjustment to base fee
        let adjusted_fee = (self.config.base_fee as f64 * (1.0 + weighted_adjustment)) as u16;

        // Clamp to min/max bounds
        adjusted_fee.max(self.config.min_fee).min(self.config.max_fee)
    }

    /// Calculate volatility-based fee adjustment
    /// Higher volatility -> higher fees to compensate LPs for impermanent loss risk
    fn calculate_volatility_factor(&self, price_history: &[PricePoint]) -> f64 {
        if price_history.len() < 2 {
            return 0.0;
        }

        // Calculate standard deviation of price changes
        let returns: Vec<f64> = price_history
            .windows(2)
            .map(|w| {
                let price_change = (w[1].price - w[0].price) / w[0].price;
                price_change
            })
            .collect();

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;

        let std_dev = variance.sqrt();

        // Annualized volatility (assuming daily data)
        let annualized_vol = std_dev * (365.0_f64).sqrt();

        // Normalize to adjustment factor
        // 0% vol = 0x adjustment, 100% vol = 1x adjustment (double fee)
        (annualized_vol * 1.0).min(1.0)
    }

    /// Calculate volume-based fee adjustment
    /// Lower volume -> higher fees to compensate for illiquidity
    fn calculate_volume_factor(&self, volume_history: &[VolumePoint]) -> f64 {
        if volume_history.is_empty() {
            return 0.0;
        }

        let total_volume: U256 = volume_history
            .iter()
            .map(|v| v.volume)
            .sum();

        let avg_volume = total_volume / U256::from(volume_history.len());

        // Calculate recent volume vs historical average
        let recent_period = volume_history.len().min(6); // Last 6 periods
        let recent_volume: U256 = volume_history
            .iter()
            .rev()
            .take(recent_period)
            .map(|v| v.volume)
            .sum();

        let recent_avg = recent_volume / U256::from(recent_period);

        // If recent volume < historical average, increase fees
        if recent_avg < avg_volume {
            let ratio = recent_avg.as_u128() as f64 / avg_volume.as_u128() as f64;
            (1.0 - ratio).min(0.5) // Max 0.5x adjustment
        } else {
            // If recent volume > average, decrease fees
            let ratio = avg_volume.as_u128() as f64 / recent_avg.as_u128() as f64;
            -(1.0 - ratio).min(0.3) // Max -0.3x adjustment (fee reduction)
        }
    }

    /// Calculate utilization-based fee adjustment
    /// Higher utilization -> higher fees to balance pool
    fn calculate_utilization_factor(&self, pool_state: &PoolState) -> f64 {
        let utilization = pool_state.calculate_utilization();

        // Sigmoid curve for smooth transitions
        // 0-50% utilization: reduce fees slightly
        // 50-80% utilization: normal fees
        // 80-100% utilization: increase fees significantly
        if utilization < 0.5 {
            -0.2 * (1.0 - utilization / 0.5)
        } else if utilization < 0.8 {
            0.0
        } else {
            2.0 * ((utilization - 0.8) / 0.2)
        }
    }

    /// Update configuration parameters
    pub fn update_config(&mut self, new_config: FeeConfig) {
        self.config = new_config;
    }
}

#[derive(Debug, Clone)]
pub struct PoolState {
    pub reserve0: U256,
    pub reserve1: U256,
    pub virtual_reserve0: U256,
    pub virtual_reserve1: U256,
    pub total_liquidity: U256,
    pub available_liquidity: U256,
}

impl PoolState {
    pub fn calculate_utilization(&self) -> f64 {
        if self.total_liquidity.is_zero() {
            return 0.0;
        }

        let utilized = self.total_liquidity - self.available_liquidity;
        utilized.as_u128() as f64 / self.total_liquidity.as_u128() as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub timestamp: u64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct VolumePoint {
    pub timestamp: u64,
    pub volume: U256,
}
```

#### 2. FeeOracle
Tracks historical data for fee calculations.

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, VecDeque};

pub struct FeeOracle {
    /// Price history by pool
    price_history: Arc<RwLock<HashMap<Address, VecDeque<PricePoint>>>>,

    /// Volume history by pool
    volume_history: Arc<RwLock<HashMap<Address, VecDeque<VolumePoint>>>>,

    /// Fee states by pool
    fee_states: Arc<RwLock<HashMap<Address, FeeState>>>,

    /// Maximum history size
    max_history_size: usize,
}

impl FeeOracle {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            price_history: Arc::new(RwLock::new(HashMap::new())),
            volume_history: Arc::new(RwLock::new(HashMap::new())),
            fee_states: Arc::new(RwLock::new(HashMap::new())),
            max_history_size,
        }
    }

    /// Record a price observation
    pub async fn record_price(&self, pool: Address, price: f64) {
        let mut history = self.price_history.write().await;
        let pool_history = history.entry(pool).or_insert_with(VecDeque::new);

        pool_history.push_back(PricePoint {
            timestamp: Self::current_timestamp(),
            price,
        });

        // Trim to max size
        while pool_history.len() > self.max_history_size {
            pool_history.pop_front();
        }
    }

    /// Record a volume observation
    pub async fn record_volume(&self, pool: Address, volume: U256) {
        let mut history = self.volume_history.write().await;
        let pool_history = history.entry(pool).or_insert_with(VecDeque::new);

        pool_history.push_back(VolumePoint {
            timestamp: Self::current_timestamp(),
            volume,
        });

        // Trim to max size
        while pool_history.len() > self.max_history_size {
            pool_history.pop_front();
        }
    }

    /// Get price history for a pool
    pub async fn get_price_history(
        &self,
        pool: Address,
        duration: u64,
    ) -> Vec<PricePoint> {
        let history = self.price_history.read().await;
        let cutoff = Self::current_timestamp() - duration;

        history
            .get(&pool)
            .map(|h| {
                h.iter()
                    .filter(|p| p.timestamp >= cutoff)
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get volume history for a pool
    pub async fn get_volume_history(
        &self,
        pool: Address,
        duration: u64,
    ) -> Vec<VolumePoint> {
        let history = self.volume_history.read().await;
        let cutoff = Self::current_timestamp() - duration;

        history
            .get(&pool)
            .map(|h| {
                h.iter()
                    .filter(|v| v.timestamp >= cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Update fee state for a pool
    pub async fn update_fee_state(&self, pool: Address, state: FeeState) {
        let mut states = self.fee_states.write().await;
        states.insert(pool, state);
    }

    /// Get current fee state
    pub async fn get_fee_state(&self, pool: Address) -> Option<FeeState> {
        let states = self.fee_states.read().await;
        states.get(&pool).cloned()
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
```

#### 3. FeeManager
Orchestrates fee calculations and updates.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FeeError {
    #[error("Pool not found: {0}")]
    PoolNotFound(Address),

    #[error("Insufficient data for fee calculation")]
    InsufficientData,

    #[error("Invalid fee configuration: {0}")]
    InvalidConfig(String),
}

pub type FeeResult<T> = std::result::Result<T, FeeError>;

pub struct FeeManager {
    calculator: FeeCalculator,
    oracle: Arc<FeeOracle>,
    config: FeeConfig,
}

impl FeeManager {
    pub fn new(config: FeeConfig) -> Self {
        Self {
            calculator: FeeCalculator::new(config.clone()),
            oracle: Arc::new(FeeOracle::new(1000)), // Store up to 1000 data points
            config,
        }
    }

    /// Get current fee for a pool
    pub async fn get_current_fee(
        &self,
        pool: Address,
        pool_state: &PoolState,
    ) -> FeeResult<u16> {
        // Check if we have cached fee state
        if let Some(state) = self.oracle.get_fee_state(pool).await {
            // Update if stale (older than 5 minutes)
            if Self::current_timestamp() - state.last_update < 300 {
                return Ok(state.current_fee);
            }
        }

        // Calculate new fee
        self.calculate_and_update_fee(pool, pool_state).await
    }

    /// Calculate and update fee for a pool
    pub async fn calculate_and_update_fee(
        &self,
        pool: Address,
        pool_state: &PoolState,
    ) -> FeeResult<u16> {
        // Get historical data
        let price_history = self.oracle
            .get_price_history(pool, self.config.volatility_window)
            .await;

        let volume_history = self.oracle
            .get_volume_history(pool, self.config.volume_window)
            .await;

        if price_history.is_empty() {
            return Err(FeeError::InsufficientData);
        }

        // Calculate fee
        let fee = self.calculator.calculate_fee(
            pool_state,
            &price_history,
            &volume_history,
        );

        // Update oracle with new state
        let volatility = if price_history.len() >= 2 {
            self.calculator.calculate_volatility_factor(&price_history)
        } else {
            0.0
        };

        let volume_24h = volume_history
            .iter()
            .map(|v| v.volume)
            .sum();

        let utilization = pool_state.calculate_utilization();

        let new_state = FeeState {
            current_fee: fee,
            volatility,
            volume_24h,
            utilization,
            last_update: Self::current_timestamp(),
            fee_history: VecDeque::new(), // Would populate from previous state
        };

        self.oracle.update_fee_state(pool, new_state).await;

        Ok(fee)
    }

    /// Record a swap for fee tracking
    pub async fn record_swap(
        &self,
        pool: Address,
        amount_in: U256,
        amount_out: U256,
        zero_for_one: bool,
    ) -> FeeResult<()> {
        // Calculate and record price
        let price = if zero_for_one {
            amount_out.as_u128() as f64 / amount_in.as_u128() as f64
        } else {
            amount_in.as_u128() as f64 / amount_out.as_u128() as f64
        };

        self.oracle.record_price(pool, price).await;
        self.oracle.record_volume(pool, amount_in).await;

        Ok(())
    }

    /// Update fee configuration
    pub async fn update_config(&mut self, new_config: FeeConfig) -> FeeResult<()> {
        // Validate config
        if new_config.min_fee > new_config.max_fee {
            return Err(FeeError::InvalidConfig(
                "min_fee cannot exceed max_fee".to_string()
            ));
        }

        if new_config.base_fee < new_config.min_fee
            || new_config.base_fee > new_config.max_fee
        {
            return Err(FeeError::InvalidConfig(
                "base_fee must be between min_fee and max_fee".to_string()
            ));
        }

        self.config = new_config.clone();
        self.calculator.update_config(new_config);

        Ok(())
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
```

## Fee Calculation Algorithm

### 1. Volatility Component
```
volatility_factor = std_dev(price_returns) * sqrt(365)
normalized_volatility = min(volatility_factor, 1.0)
```

### 2. Volume Component
```
volume_ratio = recent_volume / historical_average
if volume_ratio < 1.0:
    volume_factor = (1.0 - volume_ratio) * 0.5  // Increase fees
else:
    volume_factor = -(1.0 - 1/volume_ratio) * 0.3  // Decrease fees
```

### 3. Utilization Component
```
utilization = (total_liquidity - available_liquidity) / total_liquidity

if utilization < 0.5:
    util_factor = -0.2 * (1.0 - utilization / 0.5)
elif utilization < 0.8:
    util_factor = 0.0
else:
    util_factor = 2.0 * ((utilization - 0.8) / 0.2)
```

### 4. Final Fee Calculation
```
weighted_adjustment = (
    volatility_factor * volatility_weight +
    volume_factor * volume_weight +
    util_factor * utilization_weight
) / total_weight

adjusted_fee = base_fee * (1.0 + weighted_adjustment)
final_fee = clamp(adjusted_fee, min_fee, max_fee)
```

## Integration with Orbital AMM

### Solidity Integration
```solidity
// In OrbitalAMM contract
struct DynamicFeeState {
    uint256 base_fee;
    uint256 current_fee;
    uint256 volatility_factor;
    uint256 volume_24h;
    uint256 last_update;
    uint256 max_fee;
    uint256 min_fee;
}

function calculate_dynamic_fee(uint256 pool_id) internal returns (uint256) {
    DynamicFeeState storage fee_state = dynamic_fees[pool_id];

    // Calculate volatility from oracle data
    uint256 volatility = calculate_volatility(pool_id);

    // Calculate volume factor
    uint256 volume_factor = calculate_volume_factor(pool_id);

    // Calculate utilization
    uint256 utilization = calculate_utilization(pool_id);

    // Weighted combination
    uint256 adjustment = (
        volatility * 4000 +  // 40% weight
        volume_factor * 3000 +  // 30% weight
        utilization * 3000      // 30% weight
    ) / 10000;

    uint256 new_fee = fee_state.base_fee + (fee_state.base_fee * adjustment / 10000);

    // Clamp to bounds
    if (new_fee < fee_state.min_fee) new_fee = fee_state.min_fee;
    if (new_fee > fee_state.max_fee) new_fee = fee_state.max_fee;

    fee_state.current_fee = new_fee;
    fee_state.last_update = block.timestamp;

    return new_fee;
}
```

## Security Considerations

### 1. Fee Manipulation Prevention
- Time-weighted averages prevent single-transaction manipulation
- Maximum fee bounds protect traders
- Minimum fee bounds protect LPs

### 2. Oracle Reliability
- Multiple data sources for price/volume
- Outlier detection and filtering
- Fallback to base fee if oracle fails

### 3. Update Frequency
- Rate-limit fee updates to prevent gaming
- Smooth transitions with exponential moving average
- Emergency override for extreme conditions

## Performance Optimizations

### 1. Caching
- Cache calculated fees for 5-minute intervals
- Pre-compute common calculations
- Lazy evaluation of historical data

### 2. Batch Processing
- Batch oracle updates
- Aggregate volume across time windows
- Parallel calculation for multiple pools

### 3. Storage Efficiency
- Circular buffers for history
- Compression of historical data
- Pruning of old data points

## Testing Strategy

### Unit Tests
- Fee calculation edge cases
- Component weight adjustments
- Boundary conditions

### Integration Tests
- Oracle data flow
- Fee updates during swaps
- Configuration changes

### Simulation Tests
- Various market scenarios
- High volatility periods
- Low liquidity conditions
- Flash crash scenarios

## Monitoring and Analytics

### Key Metrics
- Average fee by time period
- Fee revenue for LPs
- Fee competitiveness vs other DEXs
- Correlation between fees and volume

### Dashboards
- Real-time fee display
- Historical fee charts
- Component breakdown (volatility/volume/utilization)
- LP profitability metrics
