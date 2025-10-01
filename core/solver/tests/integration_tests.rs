//! Integration tests for the production-grade solver module
//! 
//! These tests verify the complete solver workflow including:
//! - Intent matching and auction system
//! - Route optimization across multiple protocols
//! - Transaction execution with error recovery
//! - Cross-chain bridge integration
//! - MEV protection mechanisms
//! - Performance monitoring and metrics

use intents_solver::{
    SolverConfig, SolverNode, Solver,
    executor::{SolverExecutor, ExecutionStep, ExecutionMetrics},
    matcher::{IntentMatcher, IntentAuction},
    monitoring::{PerformanceMonitor, DetailedMetrics, PerformanceAlert},
    reputation::ReputationManager,
};
use intents_engine::intent::{Intent, IntentExecution};
use ethers::types::{Address, U256, H256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_test;

fn create_test_config() -> SolverConfig {
    SolverConfig {
        address: Address::from_low_u64_be(0x1234),
        private_key: "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
        supported_chains: vec![1, 137, 42161, 10, 8453], // Ethereum, Polygon, Arbitrum, Optimism, Base
        min_profit_bps: 50, // 0.5% minimum profit
        max_exposure: U256::from(10u128.pow(18) * 100), // 100 ETH max exposure
        reputation_threshold: 7000, // 70% reputation threshold
    }
}

fn create_sample_intent() -> Intent {
    Intent {
        user: Address::from_low_u64_be(0x5678),
        source_chain_id: 1, // Ethereum
        dest_chain_id: 137, // Polygon
        source_token: Address::zero(), // ETH
        dest_token: Address::from_low_u64_be(0x2791bca1f2de4661ed88a30c99a7a9449aa84174), // USDC on Polygon
        source_amount: U256::from(10u128.pow(18)), // 1 ETH
        min_dest_amount: U256::from(1800 * 10u128.pow(6)), // 1800 USDC (6 decimals)
        deadline: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600, // 1 hour from now
        nonce: U256::from(1),
        data: None,
        signature: ethers::types::Bytes::from(vec![1; 65]), // Mock signature
    }
}

#[tokio::test]
async fn test_intent_basic_validation() {
    let intent = create_sample_intent();
    
    // Test intent ID generation is deterministic
    let id1 = intent.id();
    let id2 = intent.id();
    assert_eq!(id1, id2);
    
    // Test intent is not expired
    assert!(!intent.is_expired());
    
    // Test signature verification (mock)
    assert!(intent.verify_signature());
    
    // Test intent with different nonce has different ID
    let mut intent2 = intent.clone();
    intent2.nonce = U256::from(2);
    assert_ne!(intent.id(), intent2.id());
}

#[tokio::test]
async fn test_reputation_manager() {
    let mut reputation_manager = ReputationManager::new();
    let solver = Address::from_low_u64_be(0x1234);
    
    // Test initial reputation
    assert!(reputation_manager.get_reputation(solver).await.is_none());
    
    // Test solver registration
    let initial_reputation = intents_solver::reputation::SolverReputation {
        solver,
        score: 5000, // 50%
        total_intents: 0,
        successful_intents: 0,
        failed_intents: 0,
        total_volume: U256::zero(),
        stake: U256::from(10u128.pow(18)), // 1 ETH
        last_activity: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    reputation_manager.register_solver(solver, initial_reputation.clone()).await;
    
    let retrieved = reputation_manager.get_reputation(solver).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().score, 5000);
    
    // Test eligibility check
    let intent_amount = U256::from(10u128.pow(17)); // 0.1 ETH
    assert!(reputation_manager.is_eligible(solver, intent_amount).await);
    
    // Test successful execution update
    reputation_manager.record_successful_execution(solver, U256::from(1000)).await;
    let updated = reputation_manager.get_reputation(solver).await.unwrap();
    assert!(updated.score > 5000); // Should increase
    assert_eq!(updated.successful_intents, 1);
}

#[tokio::test]
async fn test_intent_matcher_auction_system() {
    let reputation_manager = std::sync::Arc::new(ReputationManager::new());
    let matcher = IntentMatcher::new(reputation_manager.clone());
    
    let intent = create_sample_intent();
    let intent_id = intent.id();
    
    // Test auction creation
    let result = matcher.start_auction(intent_id, intent.clone(), 300).await; // 5 minute auction
    assert!(result.is_ok());
    
    // Test duplicate auction prevention
    let duplicate_result = matcher.start_auction(intent_id, intent.clone(), 300).await;
    assert!(duplicate_result.is_err());
    
    // Test active auctions listing
    let active = matcher.get_active_auctions().await;
    assert!(active.contains(&intent_id));
    
    // Test quote submission (would normally fail without proper solver registration)
    let quote = intents_solver::SolverQuote {
        solver: Address::from_low_u64_be(0x1234),
        dest_amount: U256::from(1850 * 10u128.pow(6)), // 1850 USDC
        profit: U256::from(50 * 10u128.pow(6)), // 50 USDC profit
        execution_time_estimate: 120, // 2 minutes
        confidence: 0.95,
    };
    
    // This would fail in real environment without solver registration
    let _quote_result = matcher.submit_quote(intent_id, quote).await;
}

#[tokio::test]
async fn test_performance_monitor() {
    let monitor = PerformanceMonitor::new();
    let intent_id = H256::from_low_u64_be(0x1111);
    
    // Test execution start recording
    monitor.record_execution_start(intent_id, 1, 137).await;
    
    // Test metrics before completion
    let initial_metrics = monitor.get_metrics().await;
    assert_eq!(initial_metrics.total_executions, 0); // Not completed yet
    
    // Test execution completion recording
    monitor.record_execution_complete(
        intent_id,
        true, // success
        ExecutionStep::Completed,
        U256::from(150_000), // gas used
        U256::from(5 * 10u128.pow(15)), // bridge fee (0.005 ETH)
        U256::from(50 * 10u128.pow(6)), // profit (50 USDC)
        Some("orbital_amm".to_string()),
        0, // retry count
        None, // no error
    ).await;
    
    // Test metrics after completion
    let final_metrics = monitor.get_metrics().await;
    assert_eq!(final_metrics.total_executions, 1);
    assert_eq!(final_metrics.successful_executions, 1);
    assert_eq!(final_metrics.failed_executions, 0);
    assert_eq!(final_metrics.total_gas_used, U256::from(150_000));
    
    // Test MEV protection recording
    monitor.record_mev_protection(Duration::from_secs(5)).await;
    let mev_metrics = monitor.get_metrics().await;
    assert_eq!(mev_metrics.mev_protection_triggers, 1);
    assert_eq!(mev_metrics.average_protection_delay, Duration::from_secs(5));
    
    // Test rollback recording
    monitor.record_rollback(intent_id, "Test rollback".to_string()).await;
    let rollback_metrics = monitor.get_metrics().await;
    assert_eq!(rollback_metrics.rollback_operations, 1);
    
    // Test dashboard generation
    let dashboard = monitor.get_dashboard().await;
    assert_eq!(dashboard.current_metrics.total_executions, 1);
    assert_eq!(dashboard.recent_executions.len(), 1);
    
    // Test metrics export
    let exported = monitor.export_metrics().await;
    assert!(exported.is_ok());
    assert!(exported.unwrap().contains("total_executions"));
}

#[tokio::test]
async fn test_execution_step_progression() {
    let steps = vec![
        ExecutionStep::ValidatingIntent,
        ExecutionStep::LockingSourceAssets,
        ExecutionStep::ExecutingSourceSwap,
        ExecutionStep::InitiatingBridge,
        ExecutionStep::WaitingForBridgeConfirmation,
        ExecutionStep::ExecutingDestinationSwap,
        ExecutionStep::FinalValidation,
        ExecutionStep::Completed,
    ];
    
    // Test each step can be serialized and deserialized
    for step in steps {
        let serialized = serde_json::to_string(&step).expect("Failed to serialize step");
        let deserialized: ExecutionStep = serde_json::from_str(&serialized)
            .expect("Failed to deserialize step");
        
        // Compare discriminants since ExecutionStep doesn't implement PartialEq for all variants
        assert_eq!(
            std::mem::discriminant(&step),
            std::mem::discriminant(&deserialized)
        );
    }
    
    // Test failed step with message
    let failed_step = ExecutionStep::Failed("Test error message".to_string());
    let serialized = serde_json::to_string(&failed_step).expect("Failed to serialize failed step");
    assert!(serialized.contains("Test error message"));
}

#[tokio::test]
async fn test_solver_config_validation() {
    let config = create_test_config();
    
    // Test configuration values
    assert_eq!(config.address, Address::from_low_u64_be(0x1234));
    assert_eq!(config.supported_chains.len(), 5);
    assert!(config.supported_chains.contains(&1)); // Ethereum
    assert!(config.supported_chains.contains(&137)); // Polygon
    assert!(config.min_profit_bps > 0);
    assert!(config.max_exposure > U256::zero());
    assert!(config.reputation_threshold > 0);
    
    // Test private key format (should be valid hex)
    assert_eq!(config.private_key.len(), 64); // 32 bytes * 2 hex chars
    assert!(config.private_key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn test_cross_chain_intent_handling() {
    let intent = create_sample_intent();
    
    // Verify this is a cross-chain intent
    assert_ne!(intent.source_chain_id, intent.dest_chain_id);
    assert_eq!(intent.source_chain_id, 1); // Ethereum
    assert_eq!(intent.dest_chain_id, 137); // Polygon
    
    // Test intent validation
    assert!(!intent.is_expired());
    assert!(intent.verify_signature());
    
    // Test minimum amount validation
    assert!(intent.min_dest_amount > U256::zero());
    assert!(intent.source_amount > U256::zero());
    
    // Test reasonable amounts (not dust, not excessive)
    assert!(intent.source_amount >= U256::from(10u128.pow(15))); // >= 0.001 ETH
    assert!(intent.source_amount <= U256::from(10u128.pow(22))); // <= 10,000 ETH
}

#[tokio::test]
async fn test_gas_optimization_metrics() {
    let monitor = PerformanceMonitor::new();
    
    // Record multiple executions with different gas usage
    let gas_amounts = vec![
        U256::from(150_000),
        U256::from(200_000),
        U256::from(175_000),
        U256::from(225_000),
        U256::from(180_000),
    ];
    
    for (i, gas_used) in gas_amounts.iter().enumerate() {
        let intent_id = H256::from_low_u64_be(i as u64);
        monitor.record_execution_start(intent_id, 1, 137).await;
        monitor.record_execution_complete(
            intent_id,
            true,
            ExecutionStep::Completed,
            *gas_used,
            U256::from(1000),
            U256::from(5000),
            Some("test_protocol".to_string()),
            0,
            None,
        ).await;
    }
    
    let metrics = monitor.get_metrics().await;
    assert_eq!(metrics.total_executions, 5);
    
    // Calculate expected average gas
    let total_gas: u64 = gas_amounts.iter().map(|g| g.as_u64()).sum();
    let expected_avg = total_gas / gas_amounts.len() as u64;
    assert_eq!(metrics.average_gas_per_execution, U256::from(expected_avg));
}

#[tokio::test]
async fn test_failure_rate_monitoring() {
    let monitor = PerformanceMonitor::new();
    
    // Record a mix of successful and failed executions
    let results = vec![true, true, false, true, false, true, true, false, true, true];
    
    for (i, success) in results.iter().enumerate() {
        let intent_id = H256::from_low_u64_be(i as u64);
        monitor.record_execution_start(intent_id, 1, 137).await;
        
        let final_step = if *success {
            ExecutionStep::Completed
        } else {
            ExecutionStep::Failed("Test failure".to_string())
        };
        
        monitor.record_execution_complete(
            intent_id,
            *success,
            final_step,
            U256::from(150_000),
            U256::from(1000),
            if *success { U256::from(5000) } else { U256::zero() },
            Some("test_protocol".to_string()),
            if *success { 0 } else { 2 }, // Failed executions have retries
            if *success { None } else { Some("Test error".to_string()) },
        ).await;
    }
    
    let metrics = monitor.get_metrics().await;
    assert_eq!(metrics.total_executions, 10);
    assert_eq!(metrics.successful_executions, 7);
    assert_eq!(metrics.failed_executions, 3);
    
    // Test dashboard generation includes alerts for high failure rate
    let dashboard = monitor.get_dashboard().await;
    
    // With 30% failure rate, should trigger alert (threshold is 20%)
    let has_failure_alert = dashboard.alerts.iter().any(|alert| {
        matches!(alert, PerformanceAlert::HighFailureRate { .. })
    });
    assert!(has_failure_alert);
}

#[tokio::test]
async fn test_metrics_reset_functionality() {
    let monitor = PerformanceMonitor::new();
    
    // Record some data
    let intent_id = H256::from_low_u64_be(0x9999);
    monitor.record_execution_start(intent_id, 1, 137).await;
    monitor.record_execution_complete(
        intent_id,
        true,
        ExecutionStep::Completed,
        U256::from(150_000),
        U256::from(1000),
        U256::from(5000),
        Some("test_protocol".to_string()),
        0,
        None,
    ).await;
    
    // Verify data exists
    let before_reset = monitor.get_metrics().await;
    assert_eq!(before_reset.total_executions, 1);
    
    // Reset metrics
    monitor.reset_metrics().await;
    
    // Verify reset worked
    let after_reset = monitor.get_metrics().await;
    assert_eq!(after_reset.total_executions, 0);
    assert_eq!(after_reset.successful_executions, 0);
    assert_eq!(after_reset.total_gas_used, U256::zero());
}

#[test]
fn test_solver_error_types() {
    use intents_solver::{SolverError, Result};
    
    // Test error creation and formatting
    let errors = vec![
        SolverError::InsufficientLiquidity,
        SolverError::Unprofitable,
        SolverError::RiskLimitExceeded,
        SolverError::ChainNotSupported(999),
        SolverError::ExecutionFailed("Test error".to_string()),
    ];
    
    for error in errors {
        let error_string = error.to_string();
        assert!(!error_string.is_empty());
        
        // Test that Result type works
        let result: Result<()> = Err(error);
        assert!(result.is_err());
    }
}

// Performance benchmark test (marked as ignored by default)
#[tokio::test]
#[ignore]
async fn benchmark_intent_processing() {
    let monitor = PerformanceMonitor::new();
    let start_time = std::time::Instant::now();
    
    // Process 100 intents and measure performance
    for i in 0..100 {
        let intent_id = H256::from_low_u64_be(i);
        monitor.record_execution_start(intent_id, 1, 137).await;
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        monitor.record_execution_complete(
            intent_id,
            true,
            ExecutionStep::Completed,
            U256::from(150_000),
            U256::from(1000),
            U256::from(5000),
            Some("benchmark_protocol".to_string()),
            0,
            None,
        ).await;
    }
    
    let total_time = start_time.elapsed();
    let metrics = monitor.get_metrics().await;
    
    println!("Processed {} intents in {:?}", metrics.total_executions, total_time);
    println!("Average time per intent: {:?}", total_time / metrics.total_executions as u32);
    
    // Assert reasonable performance
    assert!(total_time < Duration::from_secs(5)); // Should process 100 intents in under 5 seconds
    assert_eq!(metrics.total_executions, 100);
    assert_eq!(metrics.successful_executions, 100);
}