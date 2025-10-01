# Solver Reputation System

## Overview
The Solver Reputation System tracks solver performance, reliability, and trustworthiness to enable optimal solver selection for intent execution. This system is crucial for maintaining high quality of service and protecting users from malicious or unreliable solvers.

## Architecture

### Core Components

#### 1. ReputationManager
Central component managing all solver reputations.

```rust
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReputationError {
    #[error("Solver not found: {0}")]
    SolverNotFound(Address),

    #[error("Insufficient reputation: {0}")]
    InsufficientReputation(Address),

    #[error("Solver slashed: {0}")]
    SolverSlashed(Address),

    #[error("Invalid reputation update: {0}")]
    InvalidUpdate(String),
}

pub type ReputationResult<T> = std::result::Result<T, ReputationError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverReputation {
    /// Solver address
    pub solver: Address,

    /// Overall reputation score (0-10000)
    pub score: u16,

    /// Total intents matched
    pub intents_matched: u64,

    /// Total intents executed successfully
    pub intents_executed: u64,

    /// Total intents failed
    pub intents_failed: u64,

    /// Success rate (0-10000 basis points)
    pub success_rate: u16,

    /// Average execution time (seconds)
    pub avg_execution_time: u64,

    /// Total profit earned
    pub total_profit: U256,

    /// Total gas used
    pub total_gas_used: U256,

    /// Stake amount
    pub stake: U256,

    /// Slashes received
    pub slashes: u16,

    /// Last active timestamp
    pub last_active: u64,

    /// Registration timestamp
    pub registered_at: u64,

    /// Historical performance
    pub performance_history: Vec<PerformanceSnapshot>,

    /// Specializations (chain IDs)
    pub specializations: Vec<u64>,

    /// Status
    pub status: SolverStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolverStatus {
    Active,
    Inactive,
    Suspended,
    Slashed,
    Probation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: u64,
    pub score: u16,
    pub success_rate: u16,
    pub avg_execution_time: u64,
    pub volume_24h: U256,
}

impl SolverReputation {
    pub fn new(solver: Address, initial_stake: U256) -> Self {
        let now = Self::current_timestamp();

        Self {
            solver,
            score: 5000, // Start at 50%
            intents_matched: 0,
            intents_executed: 0,
            intents_failed: 0,
            success_rate: 10000, // 100% initially
            avg_execution_time: 0,
            total_profit: U256::zero(),
            total_gas_used: U256::zero(),
            stake: initial_stake,
            slashes: 0,
            last_active: now,
            registered_at: now,
            performance_history: Vec::new(),
            specializations: Vec::new(),
            status: SolverStatus::Probation, // Start on probation
        }
    }

    /// Update success rate after intent execution
    pub fn update_success_rate(&mut self) {
        if self.intents_matched == 0 {
            self.success_rate = 10000;
            return;
        }

        self.success_rate = ((self.intents_executed as u128 * 10000)
            / self.intents_matched as u128) as u16;
    }

    /// Calculate current reputation score
    pub fn calculate_score(&self) -> u16 {
        // Multi-factor reputation scoring
        let success_weight = 4000; // 40%
        let uptime_weight = 2000;  // 20%
        let speed_weight = 2000;   // 20%
        let volume_weight = 2000;  // 20%

        // Success rate component (0-10000)
        let success_component = self.success_rate;

        // Uptime component (based on days active)
        let days_active = (Self::current_timestamp() - self.registered_at) / 86400;
        let uptime_component = (days_active.min(365) as u128 * 10000 / 365) as u16;

        // Speed component (inverse of execution time)
        // Assume ideal time is 30 seconds
        let speed_component = if self.avg_execution_time == 0 {
            10000
        } else {
            let ideal_time = 30;
            let ratio = (ideal_time as f64 / self.avg_execution_time as f64).min(1.0);
            (ratio * 10000.0) as u16
        };

        // Volume component (logarithmic scale based on total executions)
        let volume_component = if self.intents_executed == 0 {
            0
        } else {
            let log_volume = (self.intents_executed as f64).ln();
            let max_log = 10.0; // ln(~22000)
            ((log_volume / max_log).min(1.0) * 10000.0) as u16
        };

        // Weighted average
        let score = (
            success_component as u32 * success_weight
            + uptime_component as u32 * uptime_weight
            + speed_component as u32 * speed_weight
            + volume_component as u32 * volume_weight
        ) / 10000;

        // Apply penalties for slashes
        let slash_penalty = self.slashes as u32 * 500; // -5% per slash
        let final_score = score.saturating_sub(slash_penalty);

        final_score.min(10000) as u16
    }

    /// Check if solver meets minimum requirements
    pub fn meets_requirements(&self, min_score: u16, min_stake: U256) -> bool {
        self.score >= min_score
            && self.stake >= min_stake
            && self.status == SolverStatus::Active
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

pub struct ReputationManager {
    /// Solver reputations
    reputations: Arc<RwLock<HashMap<Address, SolverReputation>>>,

    /// Minimum stake requirement
    min_stake: U256,

    /// Minimum reputation score for active status
    min_score: u16,

    /// Probation period (seconds)
    probation_period: u64,

    /// Slash amount per failure
    slash_amount: U256,
}

impl ReputationManager {
    pub fn new() -> Self {
        Self {
            reputations: Arc::new(RwLock::new(HashMap::new())),
            min_stake: U256::from(1000000000000000000u64), // 1 ETH
            min_score: 5000, // 50%
            probation_period: 604800, // 7 days
            slash_amount: U256::from(100000000000000000u64), // 0.1 ETH
        }
    }

    /// Register a new solver
    pub async fn register_solver(
        &self,
        solver: Address,
        stake: U256,
    ) -> ReputationResult<()> {
        if stake < self.min_stake {
            return Err(ReputationError::InvalidUpdate(
                format!("Insufficient stake: {} < {}", stake, self.min_stake)
            ));
        }

        let mut reputations = self.reputations.write().await;

        if reputations.contains_key(&solver) {
            return Err(ReputationError::InvalidUpdate(
                "Solver already registered".to_string()
            ));
        }

        let reputation = SolverReputation::new(solver, stake);
        reputations.insert(solver, reputation);

        Ok(())
    }

    /// Record successful intent execution
    pub async fn record_success(
        &self,
        solver: Address,
        intent_id: H256,
        execution_time: u64,
        profit: U256,
        gas_used: U256,
    ) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        reputation.intents_executed += 1;
        reputation.total_profit += profit;
        reputation.total_gas_used += gas_used;
        reputation.last_active = SolverReputation::current_timestamp();

        // Update average execution time (exponential moving average)
        if reputation.avg_execution_time == 0 {
            reputation.avg_execution_time = execution_time;
        } else {
            let alpha = 0.2; // 20% weight to new value
            reputation.avg_execution_time = (
                (1.0 - alpha) * reputation.avg_execution_time as f64
                + alpha * execution_time as f64
            ) as u64;
        }

        // Update success rate and score
        reputation.update_success_rate();
        reputation.score = reputation.calculate_score();

        // Check if probation period is over
        if reputation.status == SolverStatus::Probation {
            let time_since_registration =
                SolverReputation::current_timestamp() - reputation.registered_at;

            if time_since_registration >= self.probation_period
                && reputation.intents_executed >= 10
                && reputation.success_rate >= 9000 // 90%
            {
                reputation.status = SolverStatus::Active;
            }
        }

        Ok(())
    }

    /// Record failed intent execution
    pub async fn record_failure(
        &self,
        solver: Address,
        intent_id: H256,
        reason: String,
    ) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        reputation.intents_failed += 1;
        reputation.update_success_rate();
        reputation.score = reputation.calculate_score();

        // Apply slash if failure rate is high
        if reputation.success_rate < 8000 { // Below 80%
            self.apply_slash_internal(reputation)?;
        }

        Ok(())
    }

    /// Record intent matching (commitment)
    pub async fn record_match(
        &self,
        solver: Address,
        intent_id: H256,
    ) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        reputation.intents_matched += 1;
        reputation.last_active = SolverReputation::current_timestamp();

        Ok(())
    }

    /// Apply reputation slash for misconduct
    pub async fn slash_solver(
        &self,
        solver: Address,
        reason: String,
    ) -> ReputationResult<U256> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        self.apply_slash_internal(reputation)
    }

    fn apply_slash_internal(
        &self,
        reputation: &mut SolverReputation,
    ) -> ReputationResult<U256> {
        reputation.slashes += 1;

        // Deduct from stake
        let slash_amount = self.slash_amount.min(reputation.stake);
        reputation.stake -= slash_amount;

        // Recalculate score
        reputation.score = reputation.calculate_score();

        // Suspend if too many slashes
        if reputation.slashes >= 3 {
            reputation.status = SolverStatus::Suspended;
        } else if reputation.stake < self.min_stake {
            reputation.status = SolverStatus::Slashed;
        }

        Ok(slash_amount)
    }

    /// Get solver reputation
    pub async fn get_reputation(
        &self,
        solver: Address,
    ) -> ReputationResult<SolverReputation> {
        let reputations = self.reputations.read().await;
        reputations
            .get(&solver)
            .cloned()
            .ok_or(ReputationError::SolverNotFound(solver))
    }

    /// Get top solvers by reputation
    pub async fn get_top_solvers(&self, limit: usize) -> Vec<SolverReputation> {
        let reputations = self.reputations.read().await;

        let mut solvers: Vec<_> = reputations
            .values()
            .filter(|r| r.status == SolverStatus::Active)
            .cloned()
            .collect();

        solvers.sort_by(|a, b| b.score.cmp(&a.score));
        solvers.truncate(limit);

        solvers
    }

    /// Get eligible solvers for an intent
    pub async fn get_eligible_solvers(
        &self,
        chain_id: u64,
        min_score: Option<u16>,
    ) -> Vec<SolverReputation> {
        let reputations = self.reputations.read().await;
        let min_score = min_score.unwrap_or(self.min_score);

        reputations
            .values()
            .filter(|r| {
                r.status == SolverStatus::Active
                    && r.score >= min_score
                    && r.stake >= self.min_stake
                    && (r.specializations.is_empty()
                        || r.specializations.contains(&chain_id))
            })
            .cloned()
            .collect()
    }

    /// Add solver specialization
    pub async fn add_specialization(
        &self,
        solver: Address,
        chain_id: u64,
    ) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        if !reputation.specializations.contains(&chain_id) {
            reputation.specializations.push(chain_id);
        }

        Ok(())
    }

    /// Increase solver stake
    pub async fn increase_stake(
        &self,
        solver: Address,
        amount: U256,
    ) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        reputation.stake += amount;

        // Reactivate if stake is sufficient
        if reputation.status == SolverStatus::Slashed
            && reputation.stake >= self.min_stake
        {
            reputation.status = SolverStatus::Active;
        }

        Ok(())
    }

    /// Decrease solver stake (withdrawal)
    pub async fn decrease_stake(
        &self,
        solver: Address,
        amount: U256,
    ) -> ReputationResult<U256> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        // Cannot withdraw below minimum
        let withdrawable = reputation.stake.saturating_sub(self.min_stake);
        let actual_amount = amount.min(withdrawable);

        reputation.stake -= actual_amount;

        Ok(actual_amount)
    }

    /// Take performance snapshot for historical tracking
    pub async fn take_snapshot(&self, solver: Address) -> ReputationResult<()> {
        let mut reputations = self.reputations.write().await;
        let reputation = reputations
            .get_mut(&solver)
            .ok_or(ReputationError::SolverNotFound(solver))?;

        // Calculate 24h volume
        let cutoff = SolverReputation::current_timestamp() - 86400;
        let volume_24h = reputation
            .performance_history
            .iter()
            .filter(|s| s.timestamp >= cutoff)
            .map(|s| s.volume_24h)
            .sum();

        let snapshot = PerformanceSnapshot {
            timestamp: SolverReputation::current_timestamp(),
            score: reputation.score,
            success_rate: reputation.success_rate,
            avg_execution_time: reputation.avg_execution_time,
            volume_24h,
        };

        reputation.performance_history.push(snapshot);

        // Keep only last 30 days
        let retention = 30 * 86400;
        let retention_cutoff = SolverReputation::current_timestamp() - retention;
        reputation
            .performance_history
            .retain(|s| s.timestamp >= retention_cutoff);

        Ok(())
    }
}
```

#### 2. SolverSelector
Selects best solver for intent based on reputation and requirements.

```rust
pub struct SolverSelector {
    reputation_manager: Arc<ReputationManager>,
}

impl SolverSelector {
    pub fn new(reputation_manager: Arc<ReputationManager>) -> Self {
        Self {
            reputation_manager,
        }
    }

    /// Select best solver for an intent
    pub async fn select_solver(
        &self,
        intent: &Intent,
        requirements: &SelectionRequirements,
    ) -> ReputationResult<Address> {
        // Get eligible solvers
        let mut eligible = self
            .reputation_manager
            .get_eligible_solvers(intent.source_chain_id, requirements.min_score)
            .await;

        if eligible.is_empty() {
            return Err(ReputationError::InvalidUpdate(
                "No eligible solvers found".to_string()
            ));
        }

        // Filter by additional requirements
        if let Some(min_stake) = requirements.min_stake {
            eligible.retain(|s| s.stake >= min_stake);
        }

        if let Some(max_execution_time) = requirements.max_execution_time {
            eligible.retain(|s| s.avg_execution_time <= max_execution_time);
        }

        // Score solvers for this specific intent
        let mut scored: Vec<_> = eligible
            .iter()
            .map(|s| {
                let score = self.score_solver_for_intent(s, intent, requirements);
                (s.solver, score)
            })
            .collect();

        // Sort by score
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top solver
        scored
            .first()
            .map(|(addr, _)| *addr)
            .ok_or(ReputationError::InvalidUpdate(
                "No suitable solver found".to_string()
            ))
    }

    /// Score a solver for a specific intent
    fn score_solver_for_intent(
        &self,
        solver: &SolverReputation,
        intent: &Intent,
        requirements: &SelectionRequirements,
    ) -> f64 {
        // Base reputation score (0-1)
        let reputation_score = solver.score as f64 / 10000.0;

        // Speed score (faster is better)
        let speed_score = if solver.avg_execution_time == 0 {
            1.0
        } else {
            let ideal_time = requirements
                .max_execution_time
                .unwrap_or(60);
            (ideal_time as f64 / solver.avg_execution_time as f64).min(1.0)
        };

        // Specialization bonus
        let specialization_score = if solver.specializations.contains(&intent.source_chain_id)
            && solver.specializations.contains(&intent.dest_chain_id)
        {
            1.2 // 20% bonus
        } else {
            1.0
        };

        // Recent activity score
        let now = SolverReputation::current_timestamp();
        let hours_since_active = (now - solver.last_active) / 3600;
        let activity_score = if hours_since_active < 24 {
            1.0
        } else {
            0.9 // 10% penalty for inactive
        };

        // Weighted combination
        reputation_score * 0.5
            + speed_score * 0.3
            + (specialization_score - 1.0) * 0.1
            + (activity_score - 0.9) * 0.1
    }

    /// Select multiple solvers (for redundancy or auction)
    pub async fn select_multiple_solvers(
        &self,
        intent: &Intent,
        requirements: &SelectionRequirements,
        count: usize,
    ) -> ReputationResult<Vec<Address>> {
        let mut eligible = self
            .reputation_manager
            .get_eligible_solvers(intent.source_chain_id, requirements.min_score)
            .await;

        if eligible.len() < count {
            return Err(ReputationError::InvalidUpdate(
                format!("Not enough eligible solvers: {} < {}", eligible.len(), count)
            ));
        }

        // Score and sort
        let mut scored: Vec<_> = eligible
            .iter()
            .map(|s| {
                let score = self.score_solver_for_intent(s, intent, requirements);
                (s.solver, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top N
        Ok(scored.iter().take(count).map(|(addr, _)| *addr).collect())
    }
}

#[derive(Debug, Clone)]
pub struct SelectionRequirements {
    pub min_score: Option<u16>,
    pub min_stake: Option<U256>,
    pub max_execution_time: Option<u64>,
}

use intents_engine::intent::Intent;
```

## Reputation Scoring Algorithm

### Components (Weighted)

1. **Success Rate** (40%): `intents_executed / intents_matched`
2. **Uptime** (20%): Days active / 365
3. **Speed** (20%): Ideal time / avg_execution_time
4. **Volume** (20%): ln(intents_executed) / ln(22000)

### Penalties

- **Slashes**: -5% per slash
- **Low Success Rate** (<80%): Automatic slash
- **Suspension**: 3+ slashes

### Status Transitions

```
New Registration → Probation (7 days, 10+ executions, 90%+ success)
Probation → Active
Active → Suspended (3+ slashes)
Active → Slashed (stake < minimum)
Slashed → Active (stake replenished)
```

## Security Considerations

### 1. Sybil Resistance
- Minimum stake requirement
- Reputation built over time
- Probation period for new solvers

### 2. Collusion Prevention
- Multiple independent metrics
- Historical performance tracking
- Transparent scoring algorithm

### 3. Gaming Prevention
- Non-linear scoring functions
- Diminishing returns for volume
- Time-weighted components

## Testing Strategy

### Unit Tests
- Reputation calculation accuracy
- Status transitions
- Slash mechanisms

### Integration Tests
- Solver selection under various conditions
- Multi-solver scenarios
- Edge case handling

### Simulation Tests
- Large solver population
- Various success/failure patterns
- Attack scenarios
