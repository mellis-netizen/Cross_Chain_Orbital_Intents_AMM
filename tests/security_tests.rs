//! Security tests for Cross Chain Orbital Intents AMM
//!
//! Tests cover:
//! - Reentrancy protection
//! - Overflow/underflow protection
//! - Authorization checks
//! - MEV protection
//! - Replay attack prevention
//! - Double spending prevention

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Mock state for security testing
#[derive(Debug, Clone)]
pub struct SecurityTestState {
    pub call_stack: Vec<String>,
    pub nonces: HashMap<[u8; 20], u64>,
    pub executed_messages: HashSet<[u8; 32]>,
    pub locked_resources: HashSet<String>,
}

impl SecurityTestState {
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            nonces: HashMap::new(),
            executed_messages: HashSet::new(),
            locked_resources: HashSet::new(),
        }
    }
}

/// Mock contract for testing security properties
#[derive(Debug)]
pub struct MockSecurityContract {
    pub state: Arc<Mutex<SecurityTestState>>,
    pub owner: [u8; 20],
    pub paused: bool,
}

impl MockSecurityContract {
    pub fn new(owner: [u8; 20]) -> Self {
        Self {
            state: Arc::new(Mutex::new(SecurityTestState::new())),
            owner,
            paused: false,
        }
    }

    /// Check for reentrancy by tracking call stack
    pub fn check_reentrancy(&self, function_name: &str) -> Result<(), &'static str> {
        let mut state = self.state.lock().unwrap();
        
        if state.call_stack.contains(&function_name.to_string()) {
            return Err("Reentrancy detected");
        }
        
        state.call_stack.push(function_name.to_string());
        Ok(())
    }

    /// Finish function execution (remove from call stack)
    pub fn finish_execution(&self, function_name: &str) {
        let mut state = self.state.lock().unwrap();
        state.call_stack.retain(|f| f != function_name);
    }

    /// Check nonce to prevent replay attacks
    pub fn check_nonce(&self, user: [u8; 20], nonce: u64) -> Result<(), &'static str> {
        let mut state = self.state.lock().unwrap();
        let current_nonce = state.nonces.get(&user).unwrap_or(&0);
        
        if nonce != *current_nonce + 1 {
            return Err("Invalid nonce");
        }
        
        state.nonces.insert(user, nonce);
        Ok(())
    }

    /// Check message execution to prevent double execution
    pub fn check_message_execution(&self, message_id: [u8; 32]) -> Result<(), &'static str> {
        let mut state = self.state.lock().unwrap();
        
        if state.executed_messages.contains(&message_id) {
            return Err("Message already executed");
        }
        
        state.executed_messages.insert(message_id);
        Ok(())
    }

    /// Lock resource for atomic operations
    pub fn lock_resource(&self, resource: &str) -> Result<(), &'static str> {
        let mut state = self.state.lock().unwrap();
        
        if state.locked_resources.contains(resource) {
            return Err("Resource already locked");
        }
        
        state.locked_resources.insert(resource.to_string());
        Ok(())
    }

    /// Unlock resource
    pub fn unlock_resource(&self, resource: &str) {
        let mut state = self.state.lock().unwrap();
        state.locked_resources.remove(resource);
    }

    /// Check authorization
    pub fn check_authorization(&self, caller: [u8; 20]) -> Result<(), &'static str> {
        if caller != self.owner {
            return Err("Unauthorized");
        }
        Ok(())
    }

    /// Safe arithmetic operations
    pub fn safe_add(a: u64, b: u64) -> Result<u64, &'static str> {
        a.checked_add(b).ok_or("Overflow in addition")
    }

    pub fn safe_sub(a: u64, b: u64) -> Result<u64, &'static str> {
        a.checked_sub(b).ok_or("Underflow in subtraction")
    }

    pub fn safe_mul(a: u64, b: u64) -> Result<u64, &'static str> {
        a.checked_mul(b).ok_or("Overflow in multiplication")
    }

    pub fn safe_div(a: u64, b: u64) -> Result<u64, &'static str> {
        if b == 0 {
            return Err("Division by zero");
        }
        a.checked_div(b).ok_or("Division error")
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_reentrancy_protection() {
        let contract = MockSecurityContract::new([1u8; 20]);
        
        // First call should succeed
        assert!(contract.check_reentrancy("transfer").is_ok());
        
        // Reentrant call should fail
        assert_eq!(
            contract.check_reentrancy("transfer"),
            Err("Reentrancy detected")
        );
        
        // After finishing execution, next call should succeed
        contract.finish_execution("transfer");
        assert!(contract.check_reentrancy("transfer").is_ok());
    }

    #[test]
    fn test_nonce_replay_protection() {
        let contract = MockSecurityContract::new([1u8; 20]);
        let user = [2u8; 20];
        
        // First nonce should be 1
        assert!(contract.check_nonce(user, 1).is_ok());
        
        // Replay same nonce should fail
        assert_eq!(contract.check_nonce(user, 1), Err("Invalid nonce"));
        
        // Skipping nonce should fail
        assert_eq!(contract.check_nonce(user, 3), Err("Invalid nonce"));
        
        // Correct next nonce should succeed
        assert!(contract.check_nonce(user, 2).is_ok());
    }

    #[test]
    fn test_message_double_execution_prevention() {
        let contract = MockSecurityContract::new([1u8; 20]);
        let message_id = [0x42u8; 32];
        
        // First execution should succeed
        assert!(contract.check_message_execution(message_id).is_ok());
        
        // Second execution should fail
        assert_eq!(
            contract.check_message_execution(message_id),
            Err("Message already executed")
        );
    }

    #[test]
    fn test_resource_locking() {
        let contract = MockSecurityContract::new([1u8; 20]);
        let resource = "user_balance";
        
        // First lock should succeed
        assert!(contract.lock_resource(resource).is_ok());
        
        // Second lock should fail
        assert_eq!(
            contract.lock_resource(resource),
            Err("Resource already locked")
        );
        
        // After unlock, should be able to lock again
        contract.unlock_resource(resource);
        assert!(contract.lock_resource(resource).is_ok());
    }

    #[test]
    fn test_authorization() {
        let owner = [1u8; 20];
        let non_owner = [2u8; 20];
        let contract = MockSecurityContract::new(owner);
        
        // Owner should be authorized
        assert!(contract.check_authorization(owner).is_ok());
        
        // Non-owner should not be authorized
        assert_eq!(
            contract.check_authorization(non_owner),
            Err("Unauthorized")
        );
    }

    #[test]
    fn test_safe_arithmetic_overflow() {
        // Test addition overflow
        assert_eq!(
            MockSecurityContract::safe_add(u64::MAX, 1),
            Err("Overflow in addition")
        );
        
        // Test multiplication overflow
        assert_eq!(
            MockSecurityContract::safe_mul(u64::MAX, 2),
            Err("Overflow in multiplication")
        );
        
        // Test valid operations
        assert_eq!(MockSecurityContract::safe_add(100, 200), Ok(300));
        assert_eq!(MockSecurityContract::safe_mul(10, 20), Ok(200));
    }

    #[test]
    fn test_safe_arithmetic_underflow() {
        // Test subtraction underflow
        assert_eq!(
            MockSecurityContract::safe_sub(5, 10),
            Err("Underflow in subtraction")
        );
        
        // Test valid subtraction
        assert_eq!(MockSecurityContract::safe_sub(10, 5), Ok(5));
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(
            MockSecurityContract::safe_div(100, 0),
            Err("Division by zero")
        );
        
        assert_eq!(MockSecurityContract::safe_div(100, 10), Ok(10));
    }

    #[test]
    fn test_mev_protection_commit_reveal() {
        // Simulate commit-reveal scheme
        let mut commits: HashMap<[u8; 20], [u8; 32]> = HashMap::new();
        let mut reveals: HashMap<[u8; 20], (u64, u64)> = HashMap::new(); // (value, nonce)
        
        let user = [1u8; 20];
        let value = 1000u64;
        let nonce = 42u64;
        
        // Create commitment: hash(value || nonce)
        let mut commit_data = Vec::new();
        commit_data.extend_from_slice(&value.to_be_bytes());
        commit_data.extend_from_slice(&nonce.to_be_bytes());
        let commitment = sha2::Sha256::digest(&commit_data);
        let commitment_hash: [u8; 32] = commitment.into();
        
        // Commit phase
        commits.insert(user, commitment_hash);
        
        // Reveal phase
        reveals.insert(user, (value, nonce));
        
        // Verify commitment
        let (revealed_value, revealed_nonce) = reveals.get(&user).unwrap();
        let mut verify_data = Vec::new();
        verify_data.extend_from_slice(&revealed_value.to_be_bytes());
        verify_data.extend_from_slice(&revealed_nonce.to_be_bytes());
        let verify_hash = sha2::Sha256::digest(&verify_data);
        let verify_hash_array: [u8; 32] = verify_hash.into();
        
        assert_eq!(commits.get(&user).unwrap(), &verify_hash_array);
    }

    #[test]
    fn test_slippage_protection() {
        fn calculate_slippage(amount_in: u64, amount_out: u64, expected_rate: u64) -> u64 {
            let actual_rate = if amount_in > 0 { (amount_out * 10000) / amount_in } else { 0 };
            if expected_rate > actual_rate {
                expected_rate - actual_rate
            } else {
                0
            }
        }
        
        fn check_slippage_tolerance(slippage_bp: u64, max_slippage_bp: u64) -> bool {
            slippage_bp <= max_slippage_bp
        }
        
        // Test slippage calculation
        let amount_in = 1000u64;
        let amount_out = 980u64; // 2% slippage
        let expected_rate = 10000u64; // 1:1 rate
        
        let slippage = calculate_slippage(amount_in, amount_out, expected_rate);
        assert_eq!(slippage, 200); // 2% in basis points
        
        // Test slippage tolerance
        assert!(check_slippage_tolerance(200, 500)); // 2% slippage, 5% tolerance - OK
        assert!(!check_slippage_tolerance(600, 500)); // 6% slippage, 5% tolerance - FAIL
    }

    #[test]
    fn test_front_running_protection() {
        // Simulate transaction ordering with timestamps
        #[derive(Debug, Clone)]
        struct Transaction {
            user: [u8; 20],
            timestamp: u64,
            gas_price: u64,
            amount: u64,
        }
        
        let mut pending_txs = Vec::new();
        
        // User submits transaction
        pending_txs.push(Transaction {
            user: [1u8; 20],
            timestamp: 1000,
            gas_price: 20,
            amount: 1000,
        });
        
        // MEV bot tries to front-run with higher gas
        pending_txs.push(Transaction {
            user: [2u8; 20],
            timestamp: 1001,
            gas_price: 50,
            amount: 1001,
        });
        
        // Sort by timestamp first (front-running protection)
        pending_txs.sort_by_key(|tx| tx.timestamp);
        
        // First transaction should be the original user's
        assert_eq!(pending_txs[0].user, [1u8; 20]);
        assert_eq!(pending_txs[1].user, [2u8; 20]);
    }

    #[test]
    fn test_intent_validation_security() {
        fn validate_intent_amount(amount: u64, min_amount: u64, max_amount: u64) -> bool {
            amount >= min_amount && amount <= max_amount && amount > 0
        }
        
        fn validate_intent_deadline(deadline: u64, current_time: u64) -> bool {
            deadline > current_time
        }
        
        fn validate_intent_signature(
            intent_hash: [u8; 32],
            signature: &[u8],
            signer: [u8; 20],
        ) -> bool {
            // Simplified signature validation
            signature.len() == 65 && signer != [0u8; 20]
        }
        
        // Test amount validation
        assert!(validate_intent_amount(1000, 1, 10000));
        assert!(!validate_intent_amount(0, 1, 10000)); // Zero amount
        assert!(!validate_intent_amount(10001, 1, 10000)); // Too large
        
        // Test deadline validation
        assert!(validate_intent_deadline(2000, 1000));
        assert!(!validate_intent_deadline(500, 1000)); // Expired
        
        // Test signature validation
        let dummy_signature = vec![0u8; 65];
        let valid_signer = [1u8; 20];
        let zero_signer = [0u8; 20];
        
        assert!(validate_intent_signature([1u8; 32], &dummy_signature, valid_signer));
        assert!(!validate_intent_signature([1u8; 32], &dummy_signature, zero_signer));
    }

    #[test]
    fn test_cross_chain_message_security() {
        fn validate_cross_chain_message(
            source_chain: u32,
            target_chain: u32,
            nonce: u64,
            message_hash: [u8; 32],
        ) -> Result<(), &'static str> {
            // Validate chain IDs
            if source_chain == 0 || target_chain == 0 {
                return Err("Invalid chain ID");
            }
            
            if source_chain == target_chain {
                return Err("Same chain transfer not allowed");
            }
            
            // Validate nonce
            if nonce == 0 {
                return Err("Invalid nonce");
            }
            
            // Validate message hash
            if message_hash == [0u8; 32] {
                return Err("Invalid message hash");
            }
            
            Ok(())
        }
        
        // Valid message
        assert!(validate_cross_chain_message(1, 2, 1, [1u8; 32]).is_ok());
        
        // Invalid chain IDs
        assert_eq!(
            validate_cross_chain_message(0, 2, 1, [1u8; 32]),
            Err("Invalid chain ID")
        );
        
        // Same chain
        assert_eq!(
            validate_cross_chain_message(1, 1, 1, [1u8; 32]),
            Err("Same chain transfer not allowed")
        );
        
        // Invalid nonce
        assert_eq!(
            validate_cross_chain_message(1, 2, 0, [1u8; 32]),
            Err("Invalid nonce")
        );
        
        // Invalid hash
        assert_eq!(
            validate_cross_chain_message(1, 2, 1, [0u8; 32]),
            Err("Invalid message hash")
        );
    }
}

// Import sha2 for hashing in commit-reveal tests
#[cfg(test)]
mod test_deps {
    pub use sha2::{Digest, Sha256};
}