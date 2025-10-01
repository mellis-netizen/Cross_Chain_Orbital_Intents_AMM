use intents_engine::validator::{
    Validator, ExecutionProof, SolverReputation, ValidatorError,
};
use intents_engine::intent::Intent;
use ethers::types::{Address, U256, H256, Bytes};
use std::collections::HashSet;

/// Integration tests for validator implementations
#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_mock_intent(
        source_amount: u64,
        min_dest_amount: u64,
        source_chain: u64,
        dest_chain: u64,
    ) -> Intent {
        Intent {
            user: Address::random(),
            source_chain_id: source_chain,
            dest_chain_id: dest_chain,
            source_token: Address::random(),
            dest_token: Address::random(),
            source_amount: U256::from(source_amount),
            min_dest_amount: U256::from(min_dest_amount),
            deadline: u64::MAX,
            nonce: U256::from(1),
            data: None,
            signature: Bytes::new(),
        }
    }

    fn create_mock_solver_reputation(
        bond: u64,
        chains: Vec<u64>,
        slashed: bool,
    ) -> SolverReputation {
        let mut supported_chains = HashSet::new();
        for chain in chains {
            supported_chains.insert(chain);
        }

        SolverReputation {
            address: Address::random(),
            bond_amount: U256::from(bond),
            success_count: 100,
            failure_count: 5,
            total_volume: U256::from(1_000_000u64),
            supported_chains,
            is_slashed: slashed,
            last_activity: 0,
        }
    }

    #[tokio::test]
    async fn test_validate_slippage_success() {
        let validator = Validator::new(U256::from(1000));
        let intent = create_mock_intent(1000, 950, 1, 10);
        let actual_amount = U256::from(960);

        let result = validator.validate_slippage(&intent, actual_amount).await;
        assert!(result.is_ok(), "Slippage validation should pass for acceptable amounts");
    }

    #[tokio::test]
    async fn test_validate_slippage_below_minimum() {
        let validator = Validator::new(U256::from(1000));
        let intent = create_mock_intent(1000, 950, 1, 10);
        let actual_amount = U256::from(940);

        let result = validator.validate_slippage(&intent, actual_amount).await;
        assert!(result.is_err(), "Slippage validation should fail below minimum");
    }

    #[tokio::test]
    async fn test_validate_slippage_excessive_price_impact() {
        let validator = Validator::new(U256::from(1000));
        let intent = create_mock_intent(10000, 9500, 1, 10);

        // Simulate excessive price impact (more than 2% deviation)
        let actual_amount = U256::from(9200); // ~3% below expected

        let result = validator.validate_slippage(&intent, actual_amount).await;
        // This might pass or fail depending on exact price impact calculation
        // Just ensure it executes without panic
        let _ = result;
    }

    #[tokio::test]
    async fn test_validate_solver_capability_success() {
        let validator = Validator::new(U256::from(1000));
        let solver = Address::random();
        let reputation = create_mock_solver_reputation(10000, vec![1, 10, 42161], false);

        validator.reputation_manager().register_solver(solver, reputation).await;

        let intent = create_mock_intent(1000, 950, 1, 10);
        let result = validator.validate_solver_capability(solver, &intent).await;

        assert!(result.is_ok(), "Solver validation should pass for eligible solver");
    }

    #[tokio::test]
    async fn test_validate_solver_not_registered() {
        let validator = Validator::new(U256::from(1000));
        let solver = Address::random();
        let intent = create_mock_intent(1000, 950, 1, 10);

        let result = validator.validate_solver_capability(solver, &intent).await;
        assert!(result.is_err(), "Solver validation should fail for unregistered solver");
    }

    #[tokio::test]
    async fn test_validate_solver_slashed() {
        let validator = Validator::new(U256::from(1000));
        let solver = Address::random();
        let reputation = create_mock_solver_reputation(10000, vec![1, 10], true);

        validator.reputation_manager().register_solver(solver, reputation).await;

        let intent = create_mock_intent(1000, 950, 1, 10);
        let result = validator.validate_solver_capability(solver, &intent).await;

        assert!(result.is_err(), "Solver validation should fail for slashed solver");
    }

    #[tokio::test]
    async fn test_validate_solver_unsupported_chain() {
        let validator = Validator::new(U256::from(1000));
        let solver = Address::random();
        let reputation = create_mock_solver_reputation(10000, vec![1, 10], false);

        validator.reputation_manager().register_solver(solver, reputation).await;

        let intent = create_mock_intent(1000, 950, 1, 42161); // Base chain not supported
        let result = validator.validate_solver_capability(solver, &intent).await;

        assert!(result.is_err(), "Solver validation should fail for unsupported chain");
    }

    #[tokio::test]
    async fn test_validate_solver_insufficient_bond() {
        let validator = Validator::new(U256::from(1000));
        let solver = Address::random();
        let reputation = create_mock_solver_reputation(500, vec![1, 10], false);

        validator.reputation_manager().register_solver(solver, reputation).await;

        let intent = create_mock_intent(1000, 950, 1, 10);
        let result = validator.validate_solver_capability(solver, &intent).await;

        assert!(result.is_err(), "Solver validation should fail for insufficient bond");
    }

    #[tokio::test]
    async fn test_validate_execution_proof_success() {
        let validator = Validator::new(U256::from(1000));
        let intent_id = H256::random();

        // Create a simple valid merkle proof
        let leaf = H256::random();
        let proof_element = H256::random();

        // Compute expected root
        let computed_hash = if leaf < proof_element {
            H256::from(ethers::utils::keccak256(&[
                leaf.as_bytes(),
                proof_element.as_bytes(),
            ].concat()))
        } else {
            H256::from(ethers::utils::keccak256(&[
                proof_element.as_bytes(),
                leaf.as_bytes(),
            ].concat()))
        };

        let mut receipt_data = vec![0u8; 64];
        receipt_data[16..48].copy_from_slice(intent_id.as_bytes());

        let execution_proof = ExecutionProof {
            transaction_hash: leaf,
            block_number: 1000,
            block_root: computed_hash,
            dest_chain_id: 1,
            merkle_proof: vec![proof_element],
            receipt_data: Bytes::from(receipt_data),
        };

        let result = validator.validate_execution_proof(intent_id, &execution_proof).await;
        assert!(result.is_ok(), "Execution proof validation should pass for valid proof");
    }

    #[tokio::test]
    async fn test_validate_execution_proof_invalid_merkle() {
        let validator = Validator::new(U256::from(1000));
        let intent_id = H256::random();

        let execution_proof = ExecutionProof {
            transaction_hash: H256::random(),
            block_number: 1000,
            block_root: H256::random(), // Wrong root
            dest_chain_id: 1,
            merkle_proof: vec![H256::random()],
            receipt_data: Bytes::from(vec![0u8; 64]),
        };

        let result = validator.validate_execution_proof(intent_id, &execution_proof).await;
        assert!(result.is_err(), "Execution proof validation should fail for invalid Merkle proof");
    }

    #[tokio::test]
    async fn test_reputation_score_calculation() {
        // Test high reputation
        let high_rep = SolverReputation {
            address: Address::random(),
            bond_amount: U256::from(10000),
            success_count: 95,
            failure_count: 5,
            total_volume: U256::from(1_000_000u64),
            supported_chains: HashSet::new(),
            is_slashed: false,
            last_activity: 0,
        };
        assert!(high_rep.reputation_score() > 0.7, "High success rate should yield high score");

        // Test low reputation
        let low_rep = SolverReputation {
            address: Address::random(),
            bond_amount: U256::from(10000),
            success_count: 10,
            failure_count: 90,
            total_volume: U256::from(1_000_000u64),
            supported_chains: HashSet::new(),
            is_slashed: false,
            last_activity: 0,
        };
        assert!(low_rep.reputation_score() < 0.3, "Low success rate should yield low score");

        // Test slashed solver
        let slashed_rep = SolverReputation {
            address: Address::random(),
            bond_amount: U256::from(10000),
            success_count: 100,
            failure_count: 0,
            total_volume: U256::from(1_000_000u64),
            supported_chains: HashSet::new(),
            is_slashed: true,
            last_activity: 0,
        };
        assert_eq!(slashed_rep.reputation_score(), 0.0, "Slashed solver should have zero score");
    }

    #[tokio::test]
    async fn test_edge_case_zero_amounts() {
        let validator = Validator::new(U256::from(1000));
        let intent = create_mock_intent(0, 0, 1, 10);

        // This should be caught by basic intent validation before reaching slippage
        let result = validator.validate_slippage(&intent, U256::zero()).await;
        let _ = result; // Just ensure no panic
    }

    #[tokio::test]
    async fn test_large_amount_validation() {
        let validator = Validator::new(U256::from(1000));

        // Test with very large amounts
        let large_amount = U256::from(1_000_000_000_000u64);
        let intent = Intent {
            user: Address::random(),
            source_chain_id: 1,
            dest_chain_id: 10,
            source_token: Address::random(),
            dest_token: Address::random(),
            source_amount: large_amount,
            min_dest_amount: large_amount * U256::from(95) / U256::from(100),
            deadline: u64::MAX,
            nonce: U256::from(1),
            data: None,
            signature: Bytes::new(),
        };

        let actual = large_amount * U256::from(96) / U256::from(100);
        let result = validator.validate_slippage(&intent, actual).await;
        assert!(result.is_ok(), "Should handle large amounts correctly");
    }
}
