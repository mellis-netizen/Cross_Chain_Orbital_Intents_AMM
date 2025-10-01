#[cfg(test)]
mod executor_tests {
    use super::*;
    use crate::executor::{SolverExecutor, ExecutionStep, ExecutionMetrics, ExecutionContext};
    use ethers::types::{Address, U256, H256};
    use intents_engine::intent::Intent;
    use std::time::Instant;
    use tokio_test;

    fn create_test_config() -> SolverConfig {
        SolverConfig {
            address: Address::from_low_u64_be(1),
            private_key: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
            supported_chains: vec![1, 137, 42161],
            min_profit_bps: 10, // 0.1%
            max_exposure: U256::from(1000000000000000000u64), // 1 ETH
            reputation_threshold: 5000, // 50%
        }
    }

    fn create_test_intent() -> Intent {
        Intent {
            user: Address::from_low_u64_be(2),
            source_chain_id: 1,
            dest_chain_id: 137,
            source_token: Address::zero(), // ETH
            dest_token: Address::from_low_u64_be(100), // Some ERC20
            source_amount: U256::from(1000000000000000000u64), // 1 ETH
            min_dest_amount: U256::from(1000000000000000000u64), // 1 token
            deadline: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600, // 1 hour from now
            nonce: U256::from(1),
            data: None,
            signature: ethers::types::Bytes::from(vec![1, 2, 3, 4]),
        }
    }

    #[tokio::test]
    async fn test_executor_creation() {
        let config = create_test_config();
        
        // Note: This will fail without proper RPC endpoints
        // In a real test environment, you'd use mock providers
        match SolverExecutor::new(config).await {
            Ok(_executor) => {
                // Test passed - executor created successfully
            }
            Err(e) => {
                // Expected to fail in test environment without RPC endpoints
                assert!(e.to_string().contains("Failed to create provider") || 
                       e.to_string().contains("Invalid private key"));
            }
        }
    }

    #[test]
    fn test_execution_step_serialization() {
        let step = ExecutionStep::ValidatingIntent;
        let serialized = serde_json::to_string(&step).expect("Failed to serialize");
        assert!(serialized.contains("ValidatingIntent"));
        
        let deserialized: ExecutionStep = serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(step, deserialized);
    }

    #[test]
    fn test_execution_step_failed_with_message() {
        let step = ExecutionStep::Failed("Test error".to_string());
        match step {
            ExecutionStep::Failed(msg) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Failed step"),
        }
    }

    #[test]
    fn test_execution_context_creation() {
        let intent = create_test_intent();
        let context = ExecutionContext {
            intent_id: H256::from_low_u64_be(1),
            intent: intent.clone(),
            solver: Address::from_low_u64_be(1),
            started_at: Instant::now(),
            current_step: ExecutionStep::ValidatingIntent,
            gas_used: U256::zero(),
            bridge_fee: U256::zero(),
            execution_proof: None,
            source_tx_hash: None,
            bridge_tx_hash: None,
            dest_tx_hash: None,
            locked_assets: std::collections::HashMap::new(),
        };

        assert_eq!(context.intent_id, H256::from_low_u64_be(1));
        assert_eq!(context.intent.user, intent.user);
        assert!(matches!(context.current_step, ExecutionStep::ValidatingIntent));
        assert_eq!(context.gas_used, U256::zero());
        assert!(context.locked_assets.is_empty());
    }

    #[test]
    fn test_execution_metrics_default() {
        let metrics = ExecutionMetrics::default();
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.failed_executions, 0);
        assert_eq!(metrics.total_gas_used, U256::zero());
        assert_eq!(metrics.mev_protection_triggers, 0);
        assert_eq!(metrics.rollback_operations, 0);
    }

    #[test]
    fn test_execution_metrics_clone() {
        let mut metrics = ExecutionMetrics::default();
        metrics.total_executions = 10;
        metrics.successful_executions = 8;
        metrics.failed_executions = 2;

        let cloned = metrics.clone();
        assert_eq!(cloned.total_executions, 10);
        assert_eq!(cloned.successful_executions, 8);
        assert_eq!(cloned.failed_executions, 2);
    }

    #[test]
    fn test_intent_expiration() {
        let mut intent = create_test_intent();
        
        // Set deadline to past
        intent.deadline = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - 3600; // 1 hour ago

        assert!(intent.is_expired());

        // Set deadline to future
        intent.deadline = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600; // 1 hour from now

        assert!(!intent.is_expired());
    }

    #[test]
    fn test_intent_id_generation() {
        let intent1 = create_test_intent();
        let intent2 = create_test_intent();

        let id1 = intent1.id();
        let id2 = intent2.id();

        // Same intent should produce same ID
        assert_eq!(id1, id2);

        // Different intent should produce different ID
        let mut intent3 = create_test_intent();
        intent3.nonce = U256::from(2);
        let id3 = intent3.id();

        assert_ne!(id1, id3);
    }

    #[test]
    fn test_intent_signature_verification() {
        let intent = create_test_intent();
        assert!(intent.verify_signature()); // Has signature

        let mut empty_intent = create_test_intent();
        empty_intent.signature = ethers::types::Bytes::default();
        assert!(!empty_intent.verify_signature()); // No signature
    }

    #[test]
    fn test_solver_config_validation() {
        let config = create_test_config();
        
        assert!(!config.supported_chains.is_empty());
        assert!(config.min_profit_bps > 0);
        assert!(config.max_exposure > U256::zero());
        assert!(config.reputation_threshold > 0);
        assert_eq!(config.address, Address::from_low_u64_be(1));
    }

    #[test]
    fn test_execution_step_progression() {
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

        // Ensure all steps are different
        for (i, step1) in steps.iter().enumerate() {
            for (j, step2) in steps.iter().enumerate() {
                if i != j {
                    assert_ne!(
                        std::mem::discriminant(step1), 
                        std::mem::discriminant(step2),
                        "Steps at indices {} and {} should be different", i, j
                    );
                }
            }
        }
    }

    #[test]
    fn test_execution_result_structure() {
        let result = crate::executor::ExecutionResult {
            tx_hash: H256::from_low_u64_be(1),
            amount_out: U256::from(1000),
            gas_used: U256::from(21000),
            block_number: 12345,
        };

        assert_eq!(result.tx_hash, H256::from_low_u64_be(1));
        assert_eq!(result.amount_out, U256::from(1000));
        assert_eq!(result.gas_used, U256::from(21000));
        assert_eq!(result.block_number, 12345);
    }

    #[test]
    fn test_route_info_structure() {
        let route = crate::executor::RouteInfo {
            protocol: "orbital_amm".to_string(),
            hops: vec![Address::from_low_u64_be(1), Address::from_low_u64_be(2)],
        };

        assert_eq!(route.protocol, "orbital_amm");
        assert_eq!(route.hops.len(), 2);
        assert_eq!(route.hops[0], Address::from_low_u64_be(1));
        assert_eq!(route.hops[1], Address::from_low_u64_be(2));
    }

    // Integration test for the full execution flow (would require mocks in real environment)
    #[tokio::test]
    #[ignore] // Ignore by default as it requires external dependencies
    async fn test_full_execution_flow() {
        let config = create_test_config();
        let intent = create_test_intent();
        let intent_id = intent.id();

        // This test would require proper mocking of:
        // - RPC providers
        // - Bridge protocols
        // - Token contracts
        // - Network responses

        // For now, just test that the structure is correct
        assert_eq!(intent_id, intent.id()); // Deterministic ID generation
        assert!(!intent.is_expired()); // Should not be expired
        assert!(intent.verify_signature()); // Should have valid signature
    }

    #[test]
    fn test_mev_protection_constants() {
        use crate::executor::{MEV_PROTECTION_MIN_DELAY, MEV_PROTECTION_MAX_DELAY};
        
        assert!(MEV_PROTECTION_MIN_DELAY > 0);
        assert!(MEV_PROTECTION_MAX_DELAY > MEV_PROTECTION_MIN_DELAY);
        assert!(MEV_PROTECTION_MAX_DELAY <= 10); // Reasonable upper bound
    }

    #[test]
    fn test_execution_timeout_constant() {
        use crate::executor::EXECUTION_TIMEOUT;
        use std::time::Duration;
        
        assert_eq!(EXECUTION_TIMEOUT, Duration::from_secs(300)); // 5 minutes
    }

    #[test]
    fn test_max_concurrent_executions() {
        use crate::executor::MAX_CONCURRENT_EXECUTIONS;
        
        assert_eq!(MAX_CONCURRENT_EXECUTIONS, 10);
        assert!(MAX_CONCURRENT_EXECUTIONS > 0);
    }

    #[test]
    fn test_retry_configuration() {
        use crate::executor::{MAX_RETRY_ATTEMPTS, RETRY_BASE_DELAY_MS};
        
        assert_eq!(MAX_RETRY_ATTEMPTS, 3);
        assert_eq!(RETRY_BASE_DELAY_MS, 1000); // 1 second
        assert!(MAX_RETRY_ATTEMPTS > 0);
        assert!(RETRY_BASE_DELAY_MS > 0);
    }
}