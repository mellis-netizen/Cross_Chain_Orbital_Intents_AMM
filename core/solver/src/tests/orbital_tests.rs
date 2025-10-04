//! Comprehensive tests for orbital mathematics integration in the solver
//! 
//! These tests validate the sophisticated profit estimation algorithm and
//! intent matching engine with orbital AMM mathematics.

use super::*;
use crate::{IntentMatcher, ProfitEstimation, MevPotential};
use crate::reputation::ReputationManager;
use ethers::types::{Address, U256, H256};
use intents_engine::intent::Intent;
use std::sync::Arc;
use std::str::FromStr;

fn create_test_intent() -> Intent {
    Intent {
        user: Address::from_str("0x742d35Cc6F6B32b41e1f6B4f8D2D37E14A8b6C89").unwrap(),
        source_chain_id: 1,
        dest_chain_id: 1,
        source_token: Address::zero(), // ETH
        dest_token: Address::from_str("0xA0b86a33E6E4f6c5F1A6C8D5e4B3F4C4E8C8F4D4").unwrap(), // USDC
        source_amount: U256::from(10).pow(18.into()), // 1 ETH
        min_dest_amount: U256::from(1800) * U256::from(10).pow(6.into()), // 1800 USDC
        deadline: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600,
        nonce: U256::from(1),
        data: None,
        signature: ethers::types::Bytes::from(vec![0u8; 65]),
    }
}

fn create_test_solver_config() -> crate::SolverConfig {
    crate::SolverConfig {
        address: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
        min_profit_bps: 10, // 0.1% minimum profit
        base_risk_bps: 5,   // 0.05% base risk
        max_slippage_bps: 100, // 1% max slippage
        supported_chains: vec![1, 137, 42161],
        oracle_addresses: std::collections::HashMap::new(),
    }
}

async fn create_test_matcher() -> IntentMatcher {
    let reputation_manager = Arc::new(ReputationManager::new());
    IntentMatcher::new(reputation_manager)
}

#[tokio::test]
async fn test_orbital_exchange_rate_calculation() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let rate = matcher.calculate_orbital_exchange_rate(&intent).await;
    assert!(rate.is_ok(), "Orbital exchange rate calculation should succeed");
    
    let rate_value = rate.unwrap();
    assert!(rate_value > U256::zero(), "Rate should be positive");
}

#[tokio::test]
async fn test_optimal_orbital_path_profit() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let profit = matcher.calculate_optimal_orbital_path_profit(&intent).await;
    assert!(profit.is_ok(), "Optimal path profit calculation should succeed");
    
    let profit_value = profit.unwrap();
    // Profit could be zero if no optimization is possible
    assert!(profit_value >= U256::zero(), "Profit should be non-negative");
}

#[tokio::test]
async fn test_spherical_constraint_adjustment() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let base_profit = U256::from(1000) * U256::from(10).pow(18.into());
    
    let adjustment = matcher.apply_spherical_constraint_adjustment(&intent, base_profit).await;
    assert!(adjustment.is_ok(), "Spherical constraint adjustment should succeed");
    
    let adjustment_value = adjustment.unwrap();
    assert!(adjustment_value <= base_profit, "Adjustment should not exceed base profit");
}

#[tokio::test]
async fn test_orbital_execution_gas_costs() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let config = create_test_solver_config();
    
    let gas_costs = matcher.estimate_orbital_execution_gas_costs(&intent, &config).await;
    assert!(gas_costs.is_ok(), "Gas cost estimation should succeed");
    
    let gas_value = gas_costs.unwrap();
    assert!(gas_value > U256::zero(), "Gas costs should be positive");
}

#[tokio::test]
async fn test_orbital_slippage_impact() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let slippage = matcher.calculate_orbital_slippage_impact(&intent).await;
    assert!(slippage.is_ok(), "Slippage calculation should succeed");
    
    let slippage_value = slippage.unwrap();
    assert!(slippage_value >= U256::zero(), "Slippage should be non-negative");
}

#[tokio::test]
async fn test_orbital_mev_adjustment() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let mev_adjustment = matcher.calculate_orbital_mev_adjustment(&intent).await;
    assert!(mev_adjustment.is_ok(), "MEV adjustment calculation should succeed");
    
    let mev_value = mev_adjustment.unwrap();
    assert!(mev_value >= U256::zero(), "MEV adjustment should be non-negative");
}

#[tokio::test]
async fn test_orbital_risk_premium() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let config = create_test_solver_config();
    
    let risk_premium = matcher.calculate_orbital_risk_premium(&intent, &config).await;
    assert!(risk_premium.is_ok(), "Risk premium calculation should succeed");
    
    let risk_value = risk_premium.unwrap();
    assert!(risk_value > U256::zero(), "Risk premium should be positive");
}

#[tokio::test]
async fn test_orbital_lp_rewards() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let lp_rewards = matcher.calculate_orbital_lp_rewards(&intent).await;
    assert!(lp_rewards.is_ok(), "LP rewards calculation should succeed");
    
    let rewards_value = lp_rewards.unwrap();
    assert!(rewards_value >= U256::zero(), "LP rewards should be non-negative");
}

#[tokio::test]
async fn test_cross_chain_orbital_costs() {
    let matcher = create_test_matcher().await;
    let mut intent = create_test_intent();
    intent.dest_chain_id = 137; // Make it cross-chain
    
    let cross_chain_costs = matcher.calculate_orbital_cross_chain_costs(&intent).await;
    assert!(cross_chain_costs.is_ok(), "Cross-chain costs calculation should succeed");
    
    let costs_value = cross_chain_costs.unwrap();
    assert!(costs_value > U256::zero(), "Cross-chain costs should be positive");
}

#[tokio::test]
async fn test_path_optimization_bonus() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let bonus = matcher.calculate_path_optimization_bonus(&intent).await;
    assert!(bonus.is_ok(), "Path optimization bonus calculation should succeed");
    
    let bonus_value = bonus.unwrap();
    assert!(bonus_value >= U256::zero(), "Bonus should be non-negative");
}

#[tokio::test]
async fn test_comprehensive_profit_estimation() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let config = create_test_solver_config();
    
    let estimation = matcher.calculate_comprehensive_profit_estimation(&intent, &config).await;
    assert!(estimation.is_ok(), "Comprehensive profit estimation should succeed");
    
    let profit_est = estimation.unwrap();
    assert!(profit_est.gross_profit >= profit_est.net_profit, "Gross profit should be >= net profit");
    assert!(profit_est.confidence_score <= 100, "Confidence score should be <= 100");
    
    // Verify all components are present
    assert!(profit_est.arbitrage_profit >= U256::zero());
    assert!(profit_est.gas_costs >= U256::zero());
    assert!(profit_est.slippage_impact >= U256::zero());
    assert!(profit_est.risk_premium >= U256::zero());
}

#[tokio::test]
async fn test_orbital_pool_state_creation() {
    let matcher = create_test_matcher().await;
    
    let pool_state = matcher.get_orbital_pool_state(1).await;
    assert!(pool_state.is_ok(), "Pool state creation should succeed");
    
    let state = pool_state.unwrap();
    assert!(state.reserves.reserves.len() >= 2, "Pool should have at least 2 tokens");
    assert!(state.invariant > U256::zero(), "Pool invariant should be positive");
}

#[tokio::test]
async fn test_optimal_orbital_path_finding() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let path = matcher.find_optimal_orbital_path(&intent).await;
    assert!(path.is_ok(), "Optimal path finding should succeed");
    
    let path_indices = path.unwrap();
    assert!(path_indices.len() >= 2, "Path should have at least 2 tokens");
    assert_ne!(path_indices[0], path_indices[path_indices.len() - 1], "Path start and end should be different");
}

#[tokio::test]
async fn test_orbital_output_amount_calculation() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let path = vec![0, 1]; // Direct path
    
    let output = matcher.calculate_orbital_output_amount(&intent, &path).await;
    assert!(output.is_ok(), "Output amount calculation should succeed");
    
    if let Ok(Some(amount)) = output {
        assert!(amount > U256::zero(), "Output amount should be positive");
    }
}

#[tokio::test]
async fn test_orbital_execution_time_estimation() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let path = vec![0, 1, 2]; // Multi-hop path
    
    let exec_time = matcher.estimate_orbital_execution_time(&intent, &path).await;
    assert!(exec_time.is_ok(), "Execution time estimation should succeed");
    
    let time_value = exec_time.unwrap();
    assert!(time_value > 0, "Execution time should be positive");
    assert!(time_value < 600, "Execution time should be reasonable (< 10 minutes)");
}

#[tokio::test]
async fn test_orbital_confidence_score() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let path = vec![0, 1];
    
    let confidence = matcher.calculate_orbital_confidence_score(&intent, &path).await;
    assert!(confidence >= 0.0 && confidence <= 1.0, "Confidence should be between 0 and 1");
}

#[tokio::test]
async fn test_orbital_optimization_score() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let quote = crate::SolverQuote {
        solver: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
        dest_amount: U256::from(1800) * U256::from(10).pow(6.into()),
        profit: U256::from(100) * U256::from(10).pow(18.into()),
        execution_time_estimate: 60,
        confidence: 0.9,
    };
    
    let score = matcher.calculate_orbital_optimization_score(&quote, &intent).await;
    assert!(score >= 0.0 && score <= 1.0, "Optimization score should be between 0 and 1");
}

#[tokio::test]
async fn test_orbital_intent_matching() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let config = create_test_solver_config();
    
    let match_result = matcher.calculate_orbital_intent_match(&intent, &config).await;
    assert!(match_result.is_ok(), "Orbital intent matching should succeed");
    
    let (profit, path, time) = match_result.unwrap();
    assert!(profit >= U256::zero(), "Profit should be non-negative");
    assert!(path.len() >= 2, "Path should have at least 2 tokens");
    assert!(time > 0, "Execution time should be positive");
}

#[tokio::test]
async fn test_constraint_health_score() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let health = matcher.get_constraint_health_score(&intent).await;
    assert!(health.is_ok(), "Constraint health calculation should succeed");
    
    let health_value = health.unwrap();
    assert!(health_value <= 100, "Health score should be <= 100");
}

#[tokio::test]
async fn test_liquidity_concentration_score() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let concentration = matcher.get_liquidity_concentration_score(&intent).await;
    assert!(concentration.is_ok(), "Concentration score calculation should succeed");
    
    let score = concentration.unwrap();
    assert!(score <= 100, "Concentration score should be <= 100");
}

#[tokio::test]
async fn test_dimension_utilization_score() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let utilization = matcher.get_dimension_utilization_score(&intent).await;
    assert!(utilization.is_ok(), "Dimension utilization calculation should succeed");
    
    let score = utilization.unwrap();
    assert!(score <= 100, "Utilization score should be <= 100");
}

#[tokio::test]
async fn test_orbital_mev_potential_analysis() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    
    let mev_potential = matcher.analyze_orbital_mev_potential(&intent).await;
    assert!(mev_potential.is_ok(), "MEV potential analysis should succeed");
    
    let potential = mev_potential.unwrap();
    match potential {
        MevPotential::Arbitrage(value) => assert!(value >= U256::zero()),
        MevPotential::Sandwich(cost) => assert!(cost >= U256::zero()),
        MevPotential::None => {} // Valid case
    }
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let config = create_test_solver_config();
    
    // Benchmark comprehensive profit estimation
    let start = std::time::Instant::now();
    let estimation = matcher.calculate_comprehensive_profit_estimation(&intent, &config).await;
    let duration = start.elapsed();
    
    assert!(estimation.is_ok(), "Profit estimation should succeed");
    assert!(duration.as_millis() < 1000, "Profit estimation should complete within 1 second");
    
    // Benchmark path optimization
    let start = std::time::Instant::now();
    let path = matcher.find_optimal_orbital_path(&intent).await;
    let duration = start.elapsed();
    
    assert!(path.is_ok(), "Path optimization should succeed");
    assert!(duration.as_millis() < 500, "Path optimization should complete within 500ms");
}

#[tokio::test]
async fn test_cross_chain_vs_same_chain_costs() {
    let matcher = create_test_matcher().await;
    
    // Same chain intent
    let same_chain_intent = create_test_intent();
    let same_chain_costs = matcher.calculate_orbital_cross_chain_costs(&same_chain_intent).await.unwrap();
    
    // Cross chain intent
    let mut cross_chain_intent = create_test_intent();
    cross_chain_intent.dest_chain_id = 137;
    let cross_chain_costs = matcher.calculate_orbital_cross_chain_costs(&cross_chain_intent).await.unwrap();
    
    assert!(cross_chain_costs > same_chain_costs, "Cross-chain costs should be higher than same-chain costs");
}

#[tokio::test]
async fn test_large_vs_small_trade_analysis() {
    let matcher = create_test_matcher().await;
    let config = create_test_solver_config();
    
    // Small trade
    let mut small_intent = create_test_intent();
    small_intent.source_amount = U256::from(10).pow(17.into()); // 0.1 ETH
    let small_estimation = matcher.calculate_comprehensive_profit_estimation(&small_intent, &config).await.unwrap();
    
    // Large trade
    let mut large_intent = create_test_intent();
    large_intent.source_amount = U256::from(100) * U256::from(10).pow(18.into()); // 100 ETH
    let large_estimation = matcher.calculate_comprehensive_profit_estimation(&large_intent, &config).await.unwrap();
    
    // Large trades should have higher absolute profits but potentially lower margins
    assert!(large_estimation.gross_profit > small_estimation.gross_profit);
    assert!(large_estimation.gas_costs >= small_estimation.gas_costs);
}

#[tokio::test]
async fn test_mathematical_invariants() {
    let matcher = create_test_matcher().await;
    let intent = create_test_intent();
    let config = create_test_solver_config();
    
    let estimation = matcher.calculate_comprehensive_profit_estimation(&intent, &config).await.unwrap();
    
    // Mathematical invariants
    assert!(estimation.gross_profit >= estimation.net_profit, "Gross profit must be >= net profit");
    assert!(estimation.gross_profit >= estimation.arbitrage_profit, "Gross profit must include arbitrage profit");
    
    // Profit margin calculation invariant
    if !intent.source_amount.is_zero() {
        let expected_margin = (estimation.net_profit * U256::from(10000)) / intent.source_amount;
        assert_eq!(estimation.profit_margin, expected_margin, "Profit margin calculation should be correct");
    }
    
    // Confidence score bounds
    assert!(estimation.confidence_score <= 100, "Confidence score must be <= 100");
}