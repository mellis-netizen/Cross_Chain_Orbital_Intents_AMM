use crate::{Result, SolverError};
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Reputation score range: 0-10000 (basis points)
const MAX_REPUTATION: u64 = 10000;
const MIN_REPUTATION: u64 = 0;
const INITIAL_REPUTATION: u64 = 5000;

/// Slashing penalties (in basis points)
const SLASH_FAILED_EXECUTION: u64 = 100;  // 1%
const SLASH_TIMEOUT: u64 = 50;             // 0.5%
const SLASH_PARTIAL_FILL: u64 = 25;        // 0.25%
const SLASH_PROFIT_DEVIATION: u64 = 10;    // 0.1%

/// Reputation rewards (in basis points)
const REWARD_SUCCESSFUL_EXECUTION: u64 = 10;   // 0.1%
const REWARD_FAST_EXECUTION: u64 = 5;          // 0.05%
const REWARD_HIGH_PROFITABILITY: u64 = 15;     // 0.15%

/// Economic security parameters
const MIN_BOND_AMOUNT: u128 = 1_000_000_000_000_000_000; // 1 ETH in wei
const BOND_MULTIPLIER_BPS: u64 = 200; // 2% of total exposure

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverReputation {
    pub solver: Address,
    pub score: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_volume: U256,
    pub total_profit: U256,
    pub average_execution_time: u64,
    pub bond_amount: U256,
    pub slashed_amount: U256,
    pub last_execution_time: u64,
    pub registration_time: u64,
}

impl SolverReputation {
    pub fn new(solver: Address, bond_amount: U256) -> Self {
        Self {
            solver,
            score: INITIAL_REPUTATION,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_volume: U256::zero(),
            total_profit: U256::zero(),
            average_execution_time: 0,
            bond_amount,
            slashed_amount: U256::zero(),
            last_execution_time: current_timestamp(),
            registration_time: current_timestamp(),
        }
    }

    /// Calculate success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            return 1.0;
        }
        self.successful_executions as f64 / self.total_executions as f64
    }

    /// Calculate available bond after slashing
    pub fn available_bond(&self) -> U256 {
        if self.bond_amount > self.slashed_amount {
            self.bond_amount - self.slashed_amount
        } else {
            U256::zero()
        }
    }

    /// Check if solver has sufficient bond for given exposure
    pub fn has_sufficient_bond(&self, exposure: U256) -> bool {
        let required_bond = exposure * U256::from(BOND_MULTIPLIER_BPS) / U256::from(10000);
        self.available_bond() >= required_bond
    }

    /// Calculate profitability ratio (profit per unit volume)
    pub fn profitability_ratio(&self) -> f64 {
        if self.total_volume.is_zero() {
            return 0.0;
        }
        let profit_f64 = self.total_profit.as_u128() as f64;
        let volume_f64 = self.total_volume.as_u128() as f64;
        profit_f64 / volume_f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub intent_id: H256,
    pub solver: Address,
    pub success: bool,
    pub execution_time: u64,
    pub expected_output: U256,
    pub actual_output: U256,
    pub profit: U256,
    pub gas_used: U256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    pub solver: Address,
    pub reason: SlashingReason,
    pub amount: U256,
    pub intent_id: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SlashingReason {
    FailedExecution,
    Timeout,
    PartialFill,
    ProfitDeviation,
    InsufficientBond,
}

impl SlashingReason {
    pub fn penalty_bps(&self) -> u64 {
        match self {
            SlashingReason::FailedExecution => SLASH_FAILED_EXECUTION,
            SlashingReason::Timeout => SLASH_TIMEOUT,
            SlashingReason::PartialFill => SLASH_PARTIAL_FILL,
            SlashingReason::ProfitDeviation => SLASH_PROFIT_DEVIATION,
            SlashingReason::InsufficientBond => SLASH_FAILED_EXECUTION * 2,
        }
    }
}

pub struct ReputationManager {
    reputations: RwLock<HashMap<Address, SolverReputation>>,
    execution_history: RwLock<Vec<ExecutionReport>>,
    slashing_events: RwLock<Vec<SlashingEvent>>,
    min_reputation_threshold: u64,
}

impl ReputationManager {
    pub fn new() -> Self {
        Self {
            reputations: RwLock::new(HashMap::new()),
            execution_history: RwLock::new(Vec::new()),
            slashing_events: RwLock::new(Vec::new()),
            min_reputation_threshold: 3000, // 30%
        }
    }

    /// Register a new solver with initial bond
    pub async fn register_solver(&self, solver: Address, bond_amount: U256) -> Result<()> {
        if bond_amount < U256::from(MIN_BOND_AMOUNT) {
            return Err(SolverError::ExecutionFailed(
                "Insufficient bond amount".to_string()
            ));
        }

        let mut reputations = self.reputations.write().await;
        if reputations.contains_key(&solver) {
            return Err(SolverError::ExecutionFailed(
                "Solver already registered".to_string()
            ));
        }

        reputations.insert(solver, SolverReputation::new(solver, bond_amount));
        Ok(())
    }

    /// Get solver reputation
    pub async fn get_reputation(&self, solver: Address) -> Option<SolverReputation> {
        let reputations = self.reputations.read().await;
        reputations.get(&solver).cloned()
    }

    /// Get all active solvers sorted by reputation score
    pub async fn get_top_solvers(&self, limit: usize) -> Vec<SolverReputation> {
        let reputations = self.reputations.read().await;
        let mut solvers: Vec<_> = reputations.values().cloned().collect();

        // Sort by composite score: reputation * success_rate * profitability
        solvers.sort_by(|a, b| {
            let score_a = self.calculate_composite_score(a);
            let score_b = self.calculate_composite_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        solvers.into_iter().take(limit).collect()
    }

    /// Calculate composite score for solver selection
    fn calculate_composite_score(&self, reputation: &SolverReputation) -> f64 {
        let reputation_weight = 0.4;
        let success_weight = 0.3;
        let profit_weight = 0.2;
        let speed_weight = 0.1;

        let reputation_score = reputation.score as f64 / MAX_REPUTATION as f64;
        let success_score = reputation.success_rate();
        let profit_score = reputation.profitability_ratio().min(1.0);

        // Speed score: inverse of average execution time (normalized)
        let speed_score = if reputation.average_execution_time > 0 {
            1.0 / (1.0 + (reputation.average_execution_time as f64 / 100.0))
        } else {
            0.5
        };

        reputation_score * reputation_weight +
        success_score * success_weight +
        profit_score * profit_weight +
        speed_score * speed_weight
    }

    /// Check if solver is eligible to execute intents
    pub async fn is_eligible(&self, solver: Address, exposure: U256) -> bool {
        let reputations = self.reputations.read().await;

        if let Some(rep) = reputations.get(&solver) {
            // Check reputation threshold
            if rep.score < self.min_reputation_threshold {
                return false;
            }

            // Check bond sufficiency
            if !rep.has_sufficient_bond(exposure) {
                return false;
            }

            // Check recent activity (not inactive for more than 30 days)
            let days_inactive = (current_timestamp() - rep.last_execution_time) / 86400;
            if days_inactive > 30 {
                return false;
            }

            return true;
        }

        false
    }

    /// Record successful execution and update reputation
    pub async fn record_success(
        &self,
        report: ExecutionReport,
    ) -> Result<()> {
        let mut reputations = self.reputations.write().await;

        if let Some(rep) = reputations.get_mut(&report.solver) {
            // Update execution counts
            rep.total_executions += 1;
            rep.successful_executions += 1;

            // Update volume and profit
            rep.total_volume = rep.total_volume.saturating_add(report.actual_output);
            rep.total_profit = rep.total_profit.saturating_add(report.profit);

            // Update average execution time (exponential moving average)
            if rep.average_execution_time == 0 {
                rep.average_execution_time = report.execution_time;
            } else {
                rep.average_execution_time =
                    (rep.average_execution_time * 7 + report.execution_time) / 8;
            }

            rep.last_execution_time = current_timestamp();

            // Increase reputation score
            let mut reward = REWARD_SUCCESSFUL_EXECUTION;

            // Bonus for fast execution (< 30 seconds)
            if report.execution_time < 30 {
                reward += REWARD_FAST_EXECUTION;
            }

            // Bonus for high profitability
            let profit_ratio = if !report.expected_output.is_zero() {
                (report.profit.as_u128() as f64) / (report.expected_output.as_u128() as f64)
            } else {
                0.0
            };

            if profit_ratio > 0.01 { // > 1% profit
                reward += REWARD_HIGH_PROFITABILITY;
            }

            rep.score = (rep.score + reward).min(MAX_REPUTATION);
        }

        // Store execution report
        let mut history = self.execution_history.write().await;
        history.push(report);

        Ok(())
    }

    /// Record failed execution and apply slashing
    pub async fn record_failure(
        &self,
        intent_id: H256,
        solver: Address,
        reason: SlashingReason,
        exposure: U256,
    ) -> Result<()> {
        let mut reputations = self.reputations.write().await;

        if let Some(rep) = reputations.get_mut(&solver) {
            // Update execution counts
            rep.total_executions += 1;
            rep.failed_executions += 1;
            rep.last_execution_time = current_timestamp();

            // Calculate slashing amount
            let penalty_bps = reason.penalty_bps();
            let slash_amount = exposure * U256::from(penalty_bps) / U256::from(10000);
            let actual_slash = slash_amount.min(rep.available_bond());

            // Apply slashing
            rep.slashed_amount = rep.slashed_amount.saturating_add(actual_slash);

            // Decrease reputation score
            rep.score = rep.score.saturating_sub(penalty_bps);

            // Record slashing event
            let event = SlashingEvent {
                solver,
                reason,
                amount: actual_slash,
                intent_id,
                timestamp: current_timestamp(),
            };

            let mut slashing_events = self.slashing_events.write().await;
            slashing_events.push(event);
        }

        Ok(())
    }

    /// Increase solver bond
    pub async fn increase_bond(&self, solver: Address, amount: U256) -> Result<()> {
        let mut reputations = self.reputations.write().await;

        if let Some(rep) = reputations.get_mut(&solver) {
            rep.bond_amount = rep.bond_amount.saturating_add(amount);
            Ok(())
        } else {
            Err(SolverError::ExecutionFailed("Solver not found".to_string()))
        }
    }

    /// Withdraw slashed amount (for protocol treasury)
    pub async fn withdraw_slashed(&self, solver: Address) -> Result<U256> {
        let mut reputations = self.reputations.write().await;

        if let Some(rep) = reputations.get_mut(&solver) {
            let amount = rep.slashed_amount;
            rep.slashed_amount = U256::zero();

            // Reduce bond proportionally
            rep.bond_amount = if rep.bond_amount > amount {
                rep.bond_amount - amount
            } else {
                U256::zero()
            };

            Ok(amount)
        } else {
            Err(SolverError::ExecutionFailed("Solver not found".to_string()))
        }
    }

    /// Get solver ranking by composite score
    pub async fn get_solver_rank(&self, solver: Address) -> Option<usize> {
        let reputations = self.reputations.read().await;

        if !reputations.contains_key(&solver) {
            return None;
        }

        let mut scores: Vec<_> = reputations.iter()
            .map(|(addr, rep)| (*addr, self.calculate_composite_score(rep)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scores.iter().position(|(addr, _)| *addr == solver).map(|i| i + 1)
    }

    /// Calculate incentive reward distribution
    pub async fn calculate_rewards(
        &self,
        total_reward_pool: U256,
        period_start: u64,
        period_end: u64,
    ) -> HashMap<Address, U256> {
        let history = self.execution_history.read().await;

        // Filter executions in period
        let period_executions: Vec<_> = history.iter()
            .filter(|e| e.success && e.timestamp >= period_start && e.timestamp <= period_end)
            .collect();

        if period_executions.is_empty() {
            return HashMap::new();
        }

        // Calculate total weighted score
        let mut solver_scores: HashMap<Address, f64> = HashMap::new();

        for exec in period_executions {
            let reputations = self.reputations.read().await;
            if let Some(rep) = reputations.get(&exec.solver) {
                // Weight by volume * reputation_score * success_rate
                let volume_score = exec.actual_output.as_u128() as f64;
                let reputation_factor = rep.score as f64 / MAX_REPUTATION as f64;
                let success_factor = rep.success_rate();

                let weighted_score = volume_score * reputation_factor * success_factor;

                *solver_scores.entry(exec.solver).or_insert(0.0) += weighted_score;
            }
        }

        let total_score: f64 = solver_scores.values().sum();

        // Distribute rewards proportionally
        let mut rewards = HashMap::new();

        for (solver, score) in solver_scores {
            let share = score / total_score;
            let reward = U256::from((total_reward_pool.as_u128() as f64 * share) as u128);
            rewards.insert(solver, reward);
        }

        rewards
    }

    /// Get solver statistics for time period
    pub async fn get_solver_stats(
        &self,
        solver: Address,
        period_start: u64,
        period_end: u64,
    ) -> SolverStats {
        let history = self.execution_history.read().await;

        let solver_executions: Vec<_> = history.iter()
            .filter(|e| {
                e.solver == solver &&
                e.timestamp >= period_start &&
                e.timestamp <= period_end
            })
            .collect();

        let total = solver_executions.len() as u64;
        let successful = solver_executions.iter().filter(|e| e.success).count() as u64;
        let failed = total - successful;

        let total_volume = solver_executions.iter()
            .fold(U256::zero(), |acc, e| acc.saturating_add(e.actual_output));

        let total_profit = solver_executions.iter()
            .fold(U256::zero(), |acc, e| acc.saturating_add(e.profit));

        let avg_time = if total > 0 {
            solver_executions.iter()
                .map(|e| e.execution_time)
                .sum::<u64>() / total
        } else {
            0
        };

        SolverStats {
            solver,
            period_start,
            period_end,
            total_executions: total,
            successful_executions: successful,
            failed_executions: failed,
            total_volume,
            total_profit,
            average_execution_time: avg_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverStats {
    pub solver: Address,
    pub period_start: u64,
    pub period_end: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_volume: U256,
    pub total_profit: U256,
    pub average_execution_time: u64,
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_solver_registration() {
        let manager = ReputationManager::new();
        let solver = Address::random();
        let bond = U256::from(MIN_BOND_AMOUNT * 2);

        manager.register_solver(solver, bond).await.unwrap();

        let rep = manager.get_reputation(solver).await.unwrap();
        assert_eq!(rep.score, INITIAL_REPUTATION);
        assert_eq!(rep.bond_amount, bond);
    }

    #[tokio::test]
    async fn test_reputation_increase() {
        let manager = ReputationManager::new();
        let solver = Address::random();
        let bond = U256::from(MIN_BOND_AMOUNT * 2);

        manager.register_solver(solver, bond).await.unwrap();

        let report = ExecutionReport {
            intent_id: H256::random(),
            solver,
            success: true,
            execution_time: 25,
            expected_output: U256::from(1000),
            actual_output: U256::from(1010),
            profit: U256::from(10),
            gas_used: U256::from(150000),
            timestamp: current_timestamp(),
        };

        manager.record_success(report).await.unwrap();

        let rep = manager.get_reputation(solver).await.unwrap();
        assert!(rep.score > INITIAL_REPUTATION);
        assert_eq!(rep.successful_executions, 1);
    }

    #[tokio::test]
    async fn test_slashing() {
        let manager = ReputationManager::new();
        let solver = Address::random();
        let bond = U256::from(MIN_BOND_AMOUNT * 2);

        manager.register_solver(solver, bond).await.unwrap();

        let initial_rep = manager.get_reputation(solver).await.unwrap();

        manager.record_failure(
            H256::random(),
            solver,
            SlashingReason::FailedExecution,
            U256::from(1000),
        ).await.unwrap();

        let rep = manager.get_reputation(solver).await.unwrap();
        assert!(rep.score < initial_rep.score);
        assert!(rep.slashed_amount > U256::zero());
    }
}
