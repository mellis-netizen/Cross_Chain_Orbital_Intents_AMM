use crate::{intent::Intent, EngineError, Result};
use ethers::types::{Address, U256, H256, Bytes};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Validation-specific error types
#[derive(Error, Debug)]
pub enum ValidatorError {
    #[error("Slippage exceeded - expected: {expected}, actual: {actual}")]
    SlippageExceeded { expected: U256, actual: U256 },

    #[error("Excessive price impact detected")]
    ExcessivePriceImpact,

    #[error("Solver not registered")]
    SolverNotRegistered,

    #[error("Insufficient solver bond")]
    InsufficientBond,

    #[error("Unsupported chain")]
    UnsupportedChain,

    #[error("Invalid execution proof")]
    InvalidProof,

    #[error("Block not finalized")]
    BlockNotFinalized,

    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,

    #[error("Price oracle unavailable")]
    PriceOracleUnavailable,
}

impl From<ValidatorError> for EngineError {
    fn from(err: ValidatorError) -> Self {
        EngineError::InvalidIntent(err.to_string())
    }
}

/// Execution proof structure for cross-chain validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProof {
    pub transaction_hash: H256,
    pub block_number: u64,
    pub block_root: H256,
    pub dest_chain_id: u64,
    pub merkle_proof: Vec<H256>,
    pub receipt_data: Bytes,
}

/// Solver reputation and capability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverReputation {
    pub address: Address,
    pub bond_amount: U256,
    pub success_count: u64,
    pub failure_count: u64,
    pub total_volume: U256,
    pub supported_chains: HashSet<u64>,
    pub is_slashed: bool,
    pub last_activity: u64,
}

impl SolverReputation {
    pub fn reputation_score(&self) -> f64 {
        if self.is_slashed {
            return 0.0;
        }

        let total = self.success_count + self.failure_count;
        if total == 0 {
            return 0.5; // Neutral score for new solvers
        }

        let success_rate = self.success_count as f64 / total as f64;
        let volume_factor = (self.total_volume.as_u128() as f64).log10() / 20.0; // Normalize

        (success_rate * 0.7 + volume_factor.min(0.3)).min(1.0)
    }
}

/// Reputation manager for solver validation
pub struct ReputationManager {
    solvers: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<Address, SolverReputation>>>,
    min_bond: U256,
}

impl ReputationManager {
    pub fn new(min_bond: U256) -> Self {
        Self {
            solvers: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            min_bond,
        }
    }

    pub async fn get_reputation(&self, solver: Address) -> Option<SolverReputation> {
        self.solvers.read().await.get(&solver).cloned()
    }

    pub async fn is_eligible(&self, solver: Address, required_amount: U256) -> bool {
        if let Some(rep) = self.get_reputation(solver).await {
            !rep.is_slashed && rep.bond_amount >= self.min_bond && rep.bond_amount >= required_amount / 10
        } else {
            false
        }
    }

    pub async fn register_solver(&self, solver: Address, reputation: SolverReputation) {
        self.solvers.write().await.insert(solver, reputation);
    }
}

/// Bridge verifier for cross-chain proof validation
pub struct BridgeVerifier {
    finality_blocks: std::collections::HashMap<u64, u64>,
}

impl BridgeVerifier {
    pub fn new() -> Self {
        let mut finality_blocks = std::collections::HashMap::new();
        finality_blocks.insert(1, 64); // Ethereum mainnet
        finality_blocks.insert(10, 120); // Optimism
        finality_blocks.insert(8453, 120); // Base
        finality_blocks.insert(42161, 20); // Arbitrum

        Self { finality_blocks }
    }

    pub async fn verify_merkle_proof(
        &self,
        proof: &[H256],
        leaf: H256,
        root: H256,
    ) -> Result<bool> {
        if proof.is_empty() {
            return Ok(false);
        }

        let mut computed_hash = leaf;

        for proof_element in proof {
            computed_hash = if computed_hash < *proof_element {
                H256::from(ethers::utils::keccak256(&[
                    computed_hash.as_bytes(),
                    proof_element.as_bytes(),
                ].concat()))
            } else {
                H256::from(ethers::utils::keccak256(&[
                    proof_element.as_bytes(),
                    computed_hash.as_bytes(),
                ].concat()))
            };
        }

        Ok(computed_hash == root)
    }

    pub async fn verify_finality(
        &self,
        chain_id: u64,
        block_number: u64,
    ) -> Result<bool> {
        // In production, this would query the actual chain's current block
        // and check if enough blocks have passed for finality
        let required_confirmations = self.finality_blocks.get(&chain_id).unwrap_or(&12);

        // Mock implementation - in production, query actual chain
        // let current_block = get_current_block(chain_id).await?;
        // Ok(current_block >= block_number + required_confirmations)

        // For now, assume blocks are finalized
        Ok(true)
    }
}

/// Main validator structure
pub struct Validator {
    reputation_manager: ReputationManager,
    bridge_verifier: BridgeVerifier,
}

impl Validator {
    pub fn new(min_bond: U256) -> Self {
        Self {
            reputation_manager: ReputationManager::new(min_bond),
            bridge_verifier: BridgeVerifier::new(),
        }
    }

    /// Validate slippage protection with dynamic price impact checking
    pub async fn validate_slippage(
        &self,
        intent: &Intent,
        actual_amount: U256,
    ) -> Result<()> {
        // Check minimum amount requirement
        if actual_amount < intent.min_dest_amount {
            return Err(ValidatorError::SlippageExceeded {
                expected: intent.min_dest_amount,
                actual: actual_amount,
            }.into());
        }

        // Calculate price impact (2% max deviation threshold)
        let max_deviation = intent.source_amount * U256::from(200) / U256::from(10000);
        let price_impact = self.calculate_price_impact(intent, actual_amount)?;

        if price_impact > max_deviation {
            return Err(ValidatorError::ExcessivePriceImpact.into());
        }

        Ok(())
    }

    /// Validate solver capability and eligibility
    pub async fn validate_solver_capability(
        &self,
        solver: Address,
        intent: &Intent,
    ) -> Result<()> {
        // Check solver is registered and not slashed
        let reputation = self.reputation_manager
            .get_reputation(solver)
            .await
            .ok_or(ValidatorError::SolverNotRegistered)?;

        // Check if solver is slashed
        if reputation.is_slashed {
            return Err(ValidatorError::InsufficientBond.into());
        }

        // Check sufficient bond for intent size
        if !self.reputation_manager.is_eligible(solver, intent.source_amount).await {
            return Err(ValidatorError::InsufficientBond.into());
        }

        // Check solver supports required chains
        if !reputation.supported_chains.contains(&intent.source_chain_id) ||
           !reputation.supported_chains.contains(&intent.dest_chain_id) {
            return Err(ValidatorError::UnsupportedChain.into());
        }

        // Verify minimum reputation score
        let score = reputation.reputation_score();
        if score < 0.3 {
            return Err(ValidatorError::InsufficientBond.into());
        }

        Ok(())
    }

    /// Validate execution proof for cross-chain transactions
    pub async fn validate_execution_proof(
        &self,
        intent_id: H256,
        proof: &ExecutionProof,
    ) -> Result<()> {
        // Verify Merkle proof for transaction inclusion
        let verified = self.bridge_verifier.verify_merkle_proof(
            &proof.merkle_proof,
            proof.transaction_hash,
            proof.block_root,
        ).await?;

        if !verified {
            return Err(ValidatorError::InvalidMerkleProof.into());
        }

        // Verify block finality
        let finalized = self.bridge_verifier.verify_finality(
            proof.dest_chain_id,
            proof.block_number,
        ).await?;

        if !finalized {
            return Err(ValidatorError::BlockNotFinalized.into());
        }

        // Verify receipt data contains intent_id
        if !self.verify_intent_in_receipt(intent_id, &proof.receipt_data) {
            return Err(ValidatorError::InvalidProof.into());
        }

        Ok(())
    }

    /// Calculate price impact based on intent parameters
    fn calculate_price_impact(&self, intent: &Intent, actual_amount: U256) -> Result<U256> {
        // Expected rate = min_dest_amount / source_amount
        // Actual rate = actual_amount / source_amount
        // Impact = |expected_rate - actual_rate| * source_amount

        if intent.source_amount.is_zero() {
            return Ok(U256::zero());
        }

        let expected_rate = intent.min_dest_amount * U256::from(1_000_000) / intent.source_amount;
        let actual_rate = actual_amount * U256::from(1_000_000) / intent.source_amount;

        let impact = if actual_rate > expected_rate {
            actual_rate - expected_rate
        } else {
            expected_rate - actual_rate
        };

        // Convert back from rate difference to absolute impact
        Ok(impact * intent.source_amount / U256::from(1_000_000))
    }

    /// Verify intent ID is present in transaction receipt
    fn verify_intent_in_receipt(&self, intent_id: H256, receipt_data: &Bytes) -> bool {
        // In production, parse the receipt and check for IntentExecuted event
        // For now, check if intent_id bytes are present
        let intent_bytes = intent_id.as_bytes();
        receipt_data.windows(32).any(|window| window == intent_bytes)
    }

    /// Get reputation manager for external use
    pub fn reputation_manager(&self) -> &ReputationManager {
        &self.reputation_manager
    }
}

/// Legacy validation function for backward compatibility
pub fn validate_intent(intent: &Intent) -> Result<()> {
    if intent.source_amount == U256::zero() {
        return Err(EngineError::InvalidIntent("Source amount cannot be zero".to_string()));
    }

    if intent.min_dest_amount == U256::zero() {
        return Err(EngineError::InvalidIntent("Minimum destination amount cannot be zero".to_string()));
    }

    if intent.source_chain_id == intent.dest_chain_id && intent.source_token == intent.dest_token {
        return Err(EngineError::InvalidIntent("Same token swap on same chain not allowed".to_string()));
    }

    if intent.is_expired() {
        return Err(EngineError::IntentExpired);
    }

    if !intent.verify_signature() {
        return Err(EngineError::InvalidIntent("Invalid signature".to_string()));
    }

    // Basic slippage check
    validate_slippage(intent)?;

    Ok(())
}

fn validate_slippage(intent: &Intent) -> Result<()> {
    // Basic slippage validation - check if min_dest_amount is reasonable
    // (at least 50% of source amount for same-value tokens)
    if intent.min_dest_amount < intent.source_amount / U256::from(2) {
        return Err(EngineError::InvalidIntent(
            "Minimum destination amount too low - potential front-running risk".to_string()
        ));
    }

    Ok(())
}

/// Legacy validation functions for backward compatibility
pub fn validate_solver_capability(
    solver: Address,
    source_chain: u64,
    dest_chain: u64,
) -> Result<()> {
    // Basic validation - in production use Validator::validate_solver_capability
    if solver.is_zero() {
        return Err(EngineError::SolverNotFound(solver));
    }

    Ok(())
}

pub fn validate_execution_proof(
    intent_id: H256,
    proof: &[u8],
) -> Result<()> {
    // Basic validation - in production use Validator::validate_execution_proof
    if proof.is_empty() {
        return Err(EngineError::InvalidIntent("Empty proof provided".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slippage_validation() {
        let validator = Validator::new(U256::from(1000));

        let intent = create_test_intent();
        let actual_amount = intent.min_dest_amount + U256::from(100);

        assert!(validator.validate_slippage(&intent, actual_amount).await.is_ok());

        let low_amount = intent.min_dest_amount - U256::from(100);
        assert!(validator.validate_slippage(&intent, low_amount).await.is_err());
    }

    #[tokio::test]
    async fn test_solver_validation() {
        let validator = Validator::new(U256::from(1000));

        let solver = Address::random();
        let mut chains = HashSet::new();
        chains.insert(1);
        chains.insert(10);

        let reputation = SolverReputation {
            address: solver,
            bond_amount: U256::from(10000),
            success_count: 100,
            failure_count: 5,
            total_volume: U256::from(1_000_000),
            supported_chains: chains,
            is_slashed: false,
            last_activity: 0,
        };

        validator.reputation_manager.register_solver(solver, reputation).await;

        let intent = create_test_intent();
        assert!(validator.validate_solver_capability(solver, &intent).await.is_ok());
    }

    fn create_test_intent() -> Intent {
        Intent {
            user: Address::random(),
            source_chain_id: 1,
            dest_chain_id: 10,
            source_token: Address::random(),
            dest_token: Address::random(),
            source_amount: U256::from(1000),
            min_dest_amount: U256::from(950),
            deadline: u64::MAX,
            nonce: U256::from(1),
            data: None,
            signature: Bytes::new(),
        }
    }
}
