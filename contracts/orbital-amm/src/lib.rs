#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]

extern crate alloc;

use stylus_sdk::{alloy_primitives::{U256, Address}, prelude::*, ArbResult};
use alloy_sol_types::sol;

sol! {
    event PoolCreated(address indexed token0, address indexed token1, uint256 indexed poolId);
    event LiquidityAdded(uint256 indexed poolId, address indexed provider, uint256 amount0, uint256 amount1);
    event Swap(uint256 indexed poolId, address indexed trader, bool zeroForOne, uint256 amountIn, uint256 amountOut);
    event OracleUpdate(uint256 indexed poolId, uint256 price0, uint256 price1, uint256 timestamp);
    event PoolRebalanced(uint256 indexed poolId, uint256 newReserve0, uint256 newReserve1, uint256 timestamp);
    event DynamicFeeUpdated(uint256 indexed poolId, uint256 oldFee, uint256 newFee, uint256 volatility);
    event ArbitrageDetected(uint256 indexed poolId, uint256 priceDiff, uint256 timestamp);
    event CommitmentCreated(bytes32 indexed commitHash, address indexed trader, uint256 timestamp);
    event SwapRevealed(bytes32 indexed commitHash, uint256 indexed poolId, uint256 amountOut);
}

#[derive(SolidityError)]
pub enum OrbitalAMMError {
    PoolNotFound(PoolNotFound),
    InsufficientLiquidity(InsufficientLiquidity),
    InvalidAmount(InvalidAmount),
    Unauthorized(Unauthorized),
    SlippageExceeded(SlippageExceeded),
    InvalidCommitment(InvalidCommitment),
    CommitmentExpired(CommitmentExpired),
    RebalanceThresholdNotMet(RebalanceThresholdNotMet),
    ArbitrageLocked(ArbitrageLocked),
}

sol! {
    error PoolNotFound();
    error InsufficientLiquidity();
    error InvalidAmount();
    error Unauthorized();
    error SlippageExceeded();
    error InvalidCommitment();
    error CommitmentExpired();
    error RebalanceThresholdNotMet();
    error ArbitrageLocked();
}

sol_storage! {
    #[entrypoint]
    pub struct OrbitalAMM {
        mapping(uint256 => Pool) pools;
        mapping(address => mapping(address => uint256)) pool_ids;
        uint256 next_pool_id;
        address owner;
        mapping(uint256 => Oracle) oracles;
        uint256 fee_rate; // basis points
        mapping(uint256 => DynamicFeeState) dynamic_fees;
        mapping(uint256 => RebalanceState) rebalance_states;
        mapping(bytes32 => Commitment) commitments;
        mapping(uint256 => ArbitrageGuard) arbitrage_guards;
        uint256 commit_reveal_delay; // blocks
        uint256 twap_window; // seconds for TWAP calculation
    }

    pub struct Pool {
        address token0;
        address token1;
        uint256 reserve0;
        uint256 reserve1;
        uint256 virtual_reserve0;
        uint256 virtual_reserve1;
        uint256 k_last; // Constant product invariant
        uint256 cumulative_volume;
        bool active;
        uint256 total_liquidity_shares; // LP token tracking
        uint256 rebalance_threshold; // % deviation trigger (basis points)
    }

    pub struct Oracle {
        uint256 price0_cumulative;
        uint256 price1_cumulative;
        uint32 timestamp_last;
        uint256 reserve0_last;
        uint256 reserve1_last;
        uint256[] price_samples; // For TWAP calculation
        uint256[] timestamp_samples;
        uint256 sample_count;
    }

    pub struct DynamicFeeState {
        uint256 base_fee; // basis points
        uint256 current_fee; // dynamically adjusted
        uint256 volatility_factor; // scaled by 10000
        uint256 volume_24h;
        uint256 last_update;
        uint256 max_fee; // cap on dynamic fees
        uint256 min_fee; // floor on dynamic fees
    }

    pub struct RebalanceState {
        uint256 last_rebalance;
        uint256 rebalance_count;
        uint256 target_ratio; // target ratio scaled by 10000
        uint256 deviation; // current deviation from target
        bool auto_rebalance_enabled;
    }

    pub struct Commitment {
        address trader;
        bytes32 commit_hash;
        uint256 block_number;
        uint256 expiry;
        bool revealed;
        uint256 pool_id;
    }

    pub struct ArbitrageGuard {
        uint256 last_trade_block;
        uint256 last_price;
        uint256 price_deviation_threshold; // basis points
        uint256 cooldown_blocks;
        bool locked;
    }
}

#[public]
impl OrbitalAMM {
    /// Initialize the Orbital AMM with configuration parameters
    /// - owner: Contract administrator address
    /// - fee_rate: Base fee rate in basis points (e.g., 30 = 0.3%)
    pub fn initialize(&mut self, owner: Address, fee_rate: U256) -> ArbResult {
        self.owner.set(owner);
        self.fee_rate.set(fee_rate);
        self.commit_reveal_delay.set(U256::from(2)); // 2 blocks default
        self.twap_window.set(U256::from(1800)); // 30 minutes default
        Ok(())
    }

    /// Configure MEV protection parameters
    /// - commit_reveal_delay: Blocks to wait between commit and reveal
    /// - twap_window: Time window for TWAP calculation in seconds
    pub fn configure_mev_protection(
        &mut self,
        commit_reveal_delay: U256,
        twap_window: U256,
    ) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }
        self.commit_reveal_delay.set(commit_reveal_delay);
        self.twap_window.set(twap_window);
        Ok(())
    }

    pub fn create_pool(
        &mut self,
        token0: Address,
        token1: Address,
        virtual_reserve0: U256,
        virtual_reserve1: U256,
    ) -> Result<U256, OrbitalAMMError> {
        let (token0, token1) = if token0 < token1 {
            (token0, token1)
        } else {
            (token1, token0)
        };

        let pool_id_key = self.pool_ids.setter(token0).setter(token1);
        if pool_id_key.get() != U256::ZERO {
            return Err(OrbitalAMMError::InvalidAmount(InvalidAmount {}));
        }

        let pool_id = self.next_pool_id.get();
        pool_id_key.set(pool_id);

        let mut pool = self.pools.setter(pool_id);
        pool.token0.set(token0);
        pool.token1.set(token1);
        pool.virtual_reserve0.set(virtual_reserve0);
        pool.virtual_reserve1.set(virtual_reserve1);
        pool.active.set(true);
        pool.rebalance_threshold.set(U256::from(500)); // 5% default threshold

        // Initialize dynamic fee state
        let mut fee_state = self.dynamic_fees.setter(pool_id);
        fee_state.base_fee.set(self.fee_rate.get());
        fee_state.current_fee.set(self.fee_rate.get());
        fee_state.max_fee.set(U256::from(100)); // 1% max
        fee_state.min_fee.set(U256::from(5)); // 0.05% min
        fee_state.volatility_factor.set(U256::from(10000));

        // Initialize rebalance state
        let mut rebalance = self.rebalance_states.setter(pool_id);
        rebalance.auto_rebalance_enabled.set(true);
        rebalance.target_ratio.set(U256::from(10000)); // 1:1 default

        // Initialize arbitrage guard
        let mut arb_guard = self.arbitrage_guards.setter(pool_id);
        arb_guard.price_deviation_threshold.set(U256::from(50)); // 0.5%
        arb_guard.cooldown_blocks.set(U256::from(1));

        self.next_pool_id.set(pool_id + U256::from(1));

        evm::log(PoolCreated {
            token0,
            token1,
            poolId: pool_id,
        });

        Ok(pool_id)
    }

    pub fn add_liquidity(
        &mut self,
        pool_id: U256,
        amount0: U256,
        amount1: U256,
    ) -> Result<U256, OrbitalAMMError> {
        let mut pool = self.pools.setter(pool_id);
        
        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        if reserve0 == U256::ZERO || reserve1 == U256::ZERO {
            pool.reserve0.set(pool.reserve0.get() + amount0);
            pool.reserve1.set(pool.reserve1.get() + amount1);
        } else {
            let optimal_amount1 = amount0 * reserve1 / reserve0;
            if optimal_amount1 <= amount1 {
                pool.reserve0.set(pool.reserve0.get() + amount0);
                pool.reserve1.set(pool.reserve1.get() + optimal_amount1);
            } else {
                let optimal_amount0 = amount1 * reserve0 / reserve1;
                pool.reserve0.set(pool.reserve0.get() + optimal_amount0);
                pool.reserve1.set(pool.reserve1.get() + amount1);
            }
        }

        self.update_oracle(pool_id);
        self.update_k_invariant(pool_id)?;

        evm::log(LiquidityAdded {
            poolId: pool_id,
            provider: msg::sender(),
            amount0,
            amount1,
        });

        Ok(pool_id)
    }

    /// Update the constant product invariant k = x * y for the pool
    fn update_k_invariant(&mut self, pool_id: U256) -> Result<(), OrbitalAMMError> {
        let mut pool = self.pools.setter(pool_id);
        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        // Calculate k = reserve0 * reserve1 (constant product)
        let k = reserve0 * reserve1;
        pool.k_last.set(k);

        Ok(())
    }

    pub fn swap(
        &mut self,
        pool_id: U256,
        zero_for_one: bool,
        amount_in: U256,
        min_amount_out: U256,
    ) -> Result<U256, OrbitalAMMError> {
        if amount_in == U256::ZERO {
            return Err(OrbitalAMMError::InvalidAmount(InvalidAmount {}));
        }

        let mut pool = self.pools.setter(pool_id);
        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        // Check arbitrage guard for MEV protection
        self.check_arbitrage_guard(pool_id)?;

        // Calculate dynamic fee based on volatility
        let current_fee = self.calculate_dynamic_fee(pool_id)?;
        let amount_in_with_fee = amount_in * (U256::from(10000) - current_fee) / U256::from(10000);

        let amount_out = if zero_for_one {
            let numerator = amount_in_with_fee * reserve1;
            let denominator = reserve0 + amount_in_with_fee;
            numerator / denominator
        } else {
            let numerator = amount_in_with_fee * reserve0;
            let denominator = reserve1 + amount_in_with_fee;
            numerator / denominator
        };

        if amount_out < min_amount_out {
            return Err(OrbitalAMMError::SlippageExceeded(SlippageExceeded {}));
        }

        if zero_for_one {
            pool.reserve0.set(pool.reserve0.get() + amount_in);
            let new_reserve1 = pool.reserve1.get().saturating_sub(amount_out);
            pool.reserve1.set(new_reserve1);
        } else {
            pool.reserve1.set(pool.reserve1.get() + amount_in);
            let new_reserve0 = pool.reserve0.get().saturating_sub(amount_out);
            pool.reserve0.set(new_reserve0);
        }

        pool.cumulative_volume.set(pool.cumulative_volume.get() + amount_in);

        self.update_oracle(pool_id);
        self.update_k_invariant(pool_id)?;
        self.update_arbitrage_guard(pool_id, zero_for_one)?;

        // Check if rebalancing is needed
        self.check_and_rebalance(pool_id)?;

        evm::log(Swap {
            poolId: pool_id,
            trader: msg::sender(),
            zeroForOne: zero_for_one,
            amountIn: amount_in,
            amountOut: amount_out,
        });

        Ok(amount_out)
    }

    /// Calculate dynamic fee based on pool volatility and volume
    /// Returns the current fee in basis points
    fn calculate_dynamic_fee(&mut self, pool_id: U256) -> Result<U256, OrbitalAMMError> {
        let mut fee_state = self.dynamic_fees.setter(pool_id);
        let pool = self.pools.get(pool_id);

        let base_fee = fee_state.base_fee.get();
        let max_fee = fee_state.max_fee.get();
        let min_fee = fee_state.min_fee.get();

        // Calculate volatility based on recent price changes
        let oracle = self.oracles.get(pool_id);
        let time_elapsed = U256::from(block::timestamp() - oracle.timestamp_last.get());

        if time_elapsed == U256::ZERO {
            return Ok(base_fee);
        }

        // Calculate price change ratio
        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();
        let reserve0_last = oracle.reserve0_last.get();
        let reserve1_last = oracle.reserve1_last.get();

        if reserve0_last == U256::ZERO || reserve1_last == U256::ZERO {
            return Ok(base_fee);
        }

        // Calculate volatility as price deviation
        let current_price = reserve1 * U256::from(10000) / reserve0;
        let last_price = reserve1_last * U256::from(10000) / reserve0_last;

        let price_diff = if current_price > last_price {
            current_price - last_price
        } else {
            last_price - current_price
        };

        let volatility = price_diff * U256::from(10000) / last_price;

        // Adjust fee based on volatility: higher volatility = higher fee
        // fee = base_fee + (volatility / 100)
        let adjusted_fee = base_fee + (volatility / U256::from(100));

        // Clamp between min and max
        let current_fee = if adjusted_fee > max_fee {
            max_fee
        } else if adjusted_fee < min_fee {
            min_fee
        } else {
            adjusted_fee
        };

        // Update fee state
        let old_fee = fee_state.current_fee.get();
        fee_state.current_fee.set(current_fee);
        fee_state.volatility_factor.set(volatility);
        fee_state.last_update.set(U256::from(block::timestamp()));

        if current_fee != old_fee {
            evm::log(DynamicFeeUpdated {
                poolId: pool_id,
                oldFee: old_fee,
                newFee: current_fee,
                volatility,
            });
        }

        Ok(current_fee)
    }

    /// Check arbitrage guard to prevent MEV attacks
    fn check_arbitrage_guard(&self, pool_id: U256) -> Result<(), OrbitalAMMError> {
        let guard = self.arbitrage_guards.get(pool_id);

        if guard.locked.get() {
            let blocks_elapsed = U256::from(block::number()) - guard.last_trade_block.get();
            if blocks_elapsed < guard.cooldown_blocks.get() {
                return Err(OrbitalAMMError::ArbitrageLocked(ArbitrageLocked {}));
            }
        }

        Ok(())
    }

    /// Update arbitrage guard after a swap
    fn update_arbitrage_guard(&mut self, pool_id: U256, zero_for_one: bool) -> Result<(), OrbitalAMMError> {
        let pool = self.pools.get(pool_id);
        let mut guard = self.arbitrage_guards.setter(pool_id);

        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        let current_price = if zero_for_one {
            reserve1 * U256::from(10000) / reserve0
        } else {
            reserve0 * U256::from(10000) / reserve1
        };

        let last_price = guard.last_price.get();

        if last_price != U256::ZERO {
            let price_diff = if current_price > last_price {
                current_price - last_price
            } else {
                last_price - current_price
            };

            let deviation = price_diff * U256::from(10000) / last_price;

            if deviation > guard.price_deviation_threshold.get() {
                guard.locked.set(true);
                evm::log(ArbitrageDetected {
                    poolId: pool_id,
                    priceDiff: deviation,
                    timestamp: U256::from(block::timestamp()),
                });
            } else {
                guard.locked.set(false);
            }
        }

        guard.last_price.set(current_price);
        guard.last_trade_block.set(U256::from(block::number()));

        Ok(())
    }

    /// Check if pool needs rebalancing and execute if threshold is met
    fn check_and_rebalance(&mut self, pool_id: U256) -> Result<(), OrbitalAMMError> {
        let rebalance_state = self.rebalance_states.get(pool_id);

        if !rebalance_state.auto_rebalance_enabled.get() {
            return Ok(());
        }

        let pool = self.pools.get(pool_id);
        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        if reserve0 == U256::ZERO || reserve1 == U256::ZERO {
            return Ok(());
        }

        // Calculate current ratio vs target ratio
        let current_ratio = reserve0 * U256::from(10000) / reserve1;
        let target_ratio = rebalance_state.target_ratio.get();

        let deviation = if current_ratio > target_ratio {
            (current_ratio - target_ratio) * U256::from(10000) / target_ratio
        } else {
            (target_ratio - current_ratio) * U256::from(10000) / target_ratio
        };

        let threshold = pool.rebalance_threshold.get();

        if deviation > threshold {
            self.rebalance_pool(pool_id)?;
        }

        Ok(())
    }

    /// Rebalance pool reserves to maintain target ratio
    fn rebalance_pool(&mut self, pool_id: U256) -> Result<(), OrbitalAMMError> {
        let mut pool = self.pools.setter(pool_id);
        let mut rebalance_state = self.rebalance_states.setter(pool_id);

        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        // Calculate target reserves maintaining k constant
        let k = pool.k_last.get();
        let target_ratio = rebalance_state.target_ratio.get();

        // new_reserve0 / new_reserve1 = target_ratio / 10000
        // new_reserve0 * new_reserve1 = k
        // Solving: new_reserve1 = sqrt(k * 10000 / target_ratio)

        let denominator = target_ratio;
        if denominator == U256::ZERO {
            return Err(OrbitalAMMError::InvalidAmount(InvalidAmount {}));
        }

        // For simplicity, adjust virtual reserves to rebalance
        let total_value = reserve0 + reserve1; // Simplified value calculation
        let target_reserve0 = total_value * target_ratio / (target_ratio + U256::from(10000));
        let target_reserve1 = total_value - target_reserve0;

        // Update virtual reserves to achieve target
        let real_reserve0 = pool.reserve0.get();
        let real_reserve1 = pool.reserve1.get();

        if target_reserve0 > real_reserve0 {
            pool.virtual_reserve0.set(target_reserve0 - real_reserve0);
        } else {
            pool.virtual_reserve0.set(U256::ZERO);
        }

        if target_reserve1 > real_reserve1 {
            pool.virtual_reserve1.set(target_reserve1 - real_reserve1);
        } else {
            pool.virtual_reserve1.set(U256::ZERO);
        }

        // Update rebalance state
        rebalance_state.last_rebalance.set(U256::from(block::timestamp()));
        rebalance_state.rebalance_count.set(rebalance_state.rebalance_count.get() + U256::from(1));

        evm::log(PoolRebalanced {
            poolId: pool_id,
            newReserve0: target_reserve0,
            newReserve1: target_reserve1,
            timestamp: U256::from(block::timestamp()),
        });

        Ok(())
    }

    pub fn get_amount_out(
        &self,
        pool_id: U256,
        zero_for_one: bool,
        amount_in: U256,
    ) -> Result<U256, OrbitalAMMError> {
        let pool = self.pools.get(pool_id);
        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        let amount_in_with_fee = amount_in * (U256::from(10000) - self.fee_rate.get()) / U256::from(10000);

        let amount_out = if zero_for_one {
            let numerator = amount_in_with_fee * reserve1;
            let denominator = reserve0 + amount_in_with_fee;
            numerator / denominator
        } else {
            let numerator = amount_in_with_fee * reserve0;
            let denominator = reserve1 + amount_in_with_fee;
            numerator / denominator
        };

        Ok(amount_out)
    }

    fn update_oracle(&mut self, pool_id: U256) {
        let pool = self.pools.get(pool_id);
        let mut oracle = self.oracles.setter(pool_id);
        
        let time_elapsed = U256::from(block::timestamp() - oracle.timestamp_last.get());
        
        if time_elapsed > U256::ZERO && pool.reserve0.get() != U256::ZERO && pool.reserve1.get() != U256::ZERO {
            let price0_cumulative = oracle.price0_cumulative.get() + 
                (pool.reserve1.get() * time_elapsed / pool.reserve0.get());
            let price1_cumulative = oracle.price1_cumulative.get() + 
                (pool.reserve0.get() * time_elapsed / pool.reserve1.get());
            
            oracle.price0_cumulative.set(price0_cumulative);
            oracle.price1_cumulative.set(price1_cumulative);
        }
        
        oracle.timestamp_last.set(block::timestamp());
        oracle.reserve0_last.set(pool.reserve0.get());
        oracle.reserve1_last.set(pool.reserve1.get());
    }

    pub fn get_pool(&self, pool_id: U256) -> Pool {
        self.pools.get(pool_id)
    }

    pub fn get_pool_by_tokens(&self, token0: Address, token1: Address) -> Result<U256, OrbitalAMMError> {
        let (token0, token1) = if token0 < token1 {
            (token0, token1)
        } else {
            (token1, token0)
        };

        let pool_id = self.pool_ids.get(token0).get(token1);
        if pool_id == U256::ZERO {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        Ok(pool_id)
    }

    // ==================== MEV Protection: Commit-Reveal Scheme ====================

    /// Create a commitment for a future swap (MEV protection)
    /// commit_hash = keccak256(abi.encodePacked(pool_id, zero_for_one, amount_in, nonce, trader))
    pub fn create_commitment(
        &mut self,
        commit_hash: [u8; 32],
        pool_id: U256,
        expiry_blocks: U256,
    ) -> Result<(), OrbitalAMMError> {
        let commit_hash_u256 = U256::from_be_bytes(commit_hash);
        let mut commitment = self.commitments.setter(commit_hash_u256);

        // Ensure commitment doesn't already exist
        if commitment.trader.get() != Address::ZERO {
            return Err(OrbitalAMMError::InvalidCommitment(InvalidCommitment {}));
        }

        commitment.trader.set(msg::sender());
        commitment.commit_hash.set(commit_hash_u256);
        commitment.block_number.set(U256::from(block::number()));
        commitment.expiry.set(U256::from(block::number()) + expiry_blocks);
        commitment.revealed.set(false);
        commitment.pool_id.set(pool_id);

        evm::log(CommitmentCreated {
            commitHash: commit_hash_u256,
            trader: msg::sender(),
            timestamp: U256::from(block::timestamp()),
        });

        Ok(())
    }

    /// Reveal and execute committed swap (MEV protection)
    pub fn reveal_and_swap(
        &mut self,
        commit_hash: [u8; 32],
        pool_id: U256,
        zero_for_one: bool,
        amount_in: U256,
        min_amount_out: U256,
        nonce: U256,
    ) -> Result<U256, OrbitalAMMError> {
        let commit_hash_u256 = U256::from_be_bytes(commit_hash);
        let mut commitment = self.commitments.setter(commit_hash_u256);

        // Validate commitment exists and is from correct trader
        if commitment.trader.get() != msg::sender() {
            return Err(OrbitalAMMError::InvalidCommitment(InvalidCommitment {}));
        }

        if commitment.revealed.get() {
            return Err(OrbitalAMMError::InvalidCommitment(InvalidCommitment {}));
        }

        // Check expiry
        if U256::from(block::number()) > commitment.expiry.get() {
            return Err(OrbitalAMMError::CommitmentExpired(CommitmentExpired {}));
        }

        // Verify commit-reveal delay has passed
        let blocks_elapsed = U256::from(block::number()) - commitment.block_number.get();
        if blocks_elapsed < self.commit_reveal_delay.get() {
            return Err(OrbitalAMMError::InvalidCommitment(InvalidCommitment {}));
        }

        // Mark as revealed
        commitment.revealed.set(true);

        // Execute the swap
        let amount_out = self.swap(pool_id, zero_for_one, amount_in, min_amount_out)?;

        evm::log(SwapRevealed {
            commitHash: commit_hash_u256,
            poolId: pool_id,
            amountOut: amount_out,
        });

        Ok(amount_out)
    }

    // ==================== Price Discovery: TWAP (Time-Weighted Average Price) ====================

    /// Calculate TWAP (Time-Weighted Average Price) over configured window
    /// Returns price in basis points (scaled by 10000)
    pub fn get_twap(&self, pool_id: U256) -> Result<U256, OrbitalAMMError> {
        let oracle = self.oracles.get(pool_id);
        let pool = self.pools.get(pool_id);

        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        let time_elapsed = U256::from(block::timestamp()) - U256::from(oracle.timestamp_last.get());

        if time_elapsed == U256::ZERO {
            // Return current spot price if no time has elapsed
            let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
            let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

            if reserve0 == U256::ZERO {
                return Err(OrbitalAMMError::InsufficientLiquidity(InsufficientLiquidity {}));
            }

            return Ok(reserve1 * U256::from(10000) / reserve0);
        }

        // Calculate TWAP from cumulative price
        let twap_window = self.twap_window.get();
        let window_start = if time_elapsed > twap_window {
            U256::from(block::timestamp()) - twap_window
        } else {
            U256::from(oracle.timestamp_last.get())
        };

        let effective_time = U256::from(block::timestamp()) - window_start;

        if effective_time == U256::ZERO {
            let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
            let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();
            if reserve0 == U256::ZERO {
                return Err(OrbitalAMMError::InsufficientLiquidity(InsufficientLiquidity {}));
            }
            return Ok(reserve1 * U256::from(10000) / reserve0);
        }

        // TWAP = cumulative_price / time_elapsed
        let twap = oracle.price0_cumulative.get() / effective_time;

        Ok(twap)
    }

    /// Get instant spot price (for comparison with TWAP to detect arbitrage)
    pub fn get_spot_price(&self, pool_id: U256) -> Result<U256, OrbitalAMMError> {
        let pool = self.pools.get(pool_id);

        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        let reserve0 = pool.reserve0.get() + pool.virtual_reserve0.get();
        let reserve1 = pool.reserve1.get() + pool.virtual_reserve1.get();

        if reserve0 == U256::ZERO {
            return Err(OrbitalAMMError::InsufficientLiquidity(InsufficientLiquidity {}));
        }

        // Price = reserve1 / reserve0 (scaled by 10000 for precision)
        Ok(reserve1 * U256::from(10000) / reserve0)
    }

    /// Detect cross-chain arbitrage opportunities by comparing TWAP vs spot price
    /// Returns price deviation in basis points
    pub fn detect_arbitrage_opportunity(&self, pool_id: U256) -> Result<U256, OrbitalAMMError> {
        let twap = self.get_twap(pool_id)?;
        let spot_price = self.get_spot_price(pool_id)?;

        let deviation = if spot_price > twap {
            (spot_price - twap) * U256::from(10000) / twap
        } else {
            (twap - spot_price) * U256::from(10000) / twap
        };

        Ok(deviation)
    }

    // ==================== Liquidity Aggregation ====================

    /// Aggregate virtual liquidity from multiple sources
    /// This simulates cross-chain liquidity aggregation
    pub fn aggregate_virtual_liquidity(
        &mut self,
        pool_id: U256,
        additional_virtual0: U256,
        additional_virtual1: U256,
    ) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }

        let mut pool = self.pools.setter(pool_id);

        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        // Add virtual liquidity from cross-chain sources
        pool.virtual_reserve0.set(pool.virtual_reserve0.get() + additional_virtual0);
        pool.virtual_reserve1.set(pool.virtual_reserve1.get() + additional_virtual1);

        // Update k invariant with new virtual reserves
        self.update_k_invariant(pool_id)?;

        Ok(())
    }

    /// Remove virtual liquidity (e.g., when cross-chain source is no longer available)
    pub fn reduce_virtual_liquidity(
        &mut self,
        pool_id: U256,
        reduce_virtual0: U256,
        reduce_virtual1: U256,
    ) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }

        let mut pool = self.pools.setter(pool_id);

        if !pool.active.get() {
            return Err(OrbitalAMMError::PoolNotFound(PoolNotFound {}));
        }

        // Reduce virtual liquidity safely
        pool.virtual_reserve0.set(pool.virtual_reserve0.get().saturating_sub(reduce_virtual0));
        pool.virtual_reserve1.set(pool.virtual_reserve1.get().saturating_sub(reduce_virtual1));

        self.update_k_invariant(pool_id)?;

        Ok(())
    }

    // ==================== Advanced Configuration ====================

    /// Configure dynamic fee parameters for a pool
    pub fn configure_dynamic_fees(
        &mut self,
        pool_id: U256,
        base_fee: U256,
        min_fee: U256,
        max_fee: U256,
    ) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }

        let mut fee_state = self.dynamic_fees.setter(pool_id);
        fee_state.base_fee.set(base_fee);
        fee_state.min_fee.set(min_fee);
        fee_state.max_fee.set(max_fee);

        Ok(())
    }

    /// Configure rebalancing parameters
    pub fn configure_rebalancing(
        &mut self,
        pool_id: U256,
        threshold: U256,
        target_ratio: U256,
        auto_enabled: bool,
    ) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }

        let mut pool = self.pools.setter(pool_id);
        pool.rebalance_threshold.set(threshold);

        let mut rebalance_state = self.rebalance_states.setter(pool_id);
        rebalance_state.target_ratio.set(target_ratio);
        rebalance_state.auto_rebalance_enabled.set(auto_enabled);

        Ok(())
    }

    /// Configure arbitrage guard parameters
    pub fn configure_arbitrage_guard(
        &mut self,
        pool_id: U256,
        deviation_threshold: U256,
        cooldown_blocks: U256,
    ) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }

        let mut guard = self.arbitrage_guards.setter(pool_id);
        guard.price_deviation_threshold.set(deviation_threshold);
        guard.cooldown_blocks.set(cooldown_blocks);

        Ok(())
    }

    /// Manually trigger pool rebalancing
    pub fn manual_rebalance(&mut self, pool_id: U256) -> Result<(), OrbitalAMMError> {
        if msg::sender() != self.owner.get() {
            return Err(OrbitalAMMError::Unauthorized(Unauthorized {}));
        }

        self.rebalance_pool(pool_id)
    }

    // ==================== View Functions ====================

    /// Get comprehensive pool state including virtual reserves and metrics
    pub fn get_pool_state(&self, pool_id: U256) -> (U256, U256, U256, U256, U256, U256) {
        let pool = self.pools.get(pool_id);
        let reserve0 = pool.reserve0.get();
        let reserve1 = pool.reserve1.get();
        let virtual0 = pool.virtual_reserve0.get();
        let virtual1 = pool.virtual_reserve1.get();
        let k = pool.k_last.get();
        let volume = pool.cumulative_volume.get();

        (reserve0, reserve1, virtual0, virtual1, k, volume)
    }

    /// Get dynamic fee state for a pool
    pub fn get_fee_state(&self, pool_id: U256) -> (U256, U256, U256, U256) {
        let fee_state = self.dynamic_fees.get(pool_id);
        (
            fee_state.current_fee.get(),
            fee_state.base_fee.get(),
            fee_state.volatility_factor.get(),
            fee_state.volume_24h.get(),
        )
    }

    /// Get rebalancing state for a pool
    pub fn get_rebalance_state(&self, pool_id: U256) -> (U256, U256, U256, bool) {
        let rebalance = self.rebalance_states.get(pool_id);
        (
            rebalance.last_rebalance.get(),
            rebalance.rebalance_count.get(),
            rebalance.target_ratio.get(),
            rebalance.auto_rebalance_enabled.get(),
        )
    }

    /// Get arbitrage guard state
    pub fn get_arbitrage_guard_state(&self, pool_id: U256) -> (U256, U256, bool) {
        let guard = self.arbitrage_guards.get(pool_id);
        (
            guard.last_price.get(),
            guard.price_deviation_threshold.get(),
            guard.locked.get(),
        )
    }
}