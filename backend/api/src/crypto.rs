use ethers::{
    prelude::*,
    types::{Address, Signature, H256, U256},
    utils::{hash_message, keccak256},
};
use serde::Serialize;
use std::str::FromStr;
use secp256k1::{ecdsa::RecoveryId, Message as Secp256k1Message, PublicKey, Secp256k1};
use k256::ecdsa::{signature::Verifier, Signature as K256Signature, VerifyingKey};
use sha2::{Digest, Sha256};
use constant_time_eq::constant_time_eq;
use hex;

use crate::error::{Result, ApiError};

/// Verify an Ethereum signature for a given message and expected signer address
/// This implementation uses multiple verification methods for enhanced security
pub fn verify_signature(
    message: &[u8],
    signature: &str,
    expected_signer: Address,
) -> Result<bool> {
    // Input validation
    validate_signature_inputs(message, signature, expected_signer)?;
    
    // Primary verification using ethers
    let ethers_result = verify_signature_ethers(message, signature, expected_signer)?;
    
    // Secondary verification using secp256k1 for critical operations
    let secp256k1_result = verify_signature_secp256k1(message, signature, expected_signer)?;
    
    // Both verifications must agree
    if ethers_result != secp256k1_result {
        tracing::error!("Signature verification mismatch between ethers and secp256k1");
        return Err(ApiError::Validation("Signature verification inconsistency detected".to_string()));
    }
    
    Ok(ethers_result)
}

/// Primary signature verification using ethers library
fn verify_signature_ethers(
    message: &[u8],
    signature: &str,
    expected_signer: Address,
) -> Result<bool> {
    // Parse signature from hex string
    let sig = Signature::from_str(signature)
        .map_err(|e| ApiError::Validation(format!("Invalid signature format: {}", e)))?;
    
    // Hash the message using EIP-191 personal sign format
    let message_hash = hash_message(message);
    
    // Recover the signer address from the signature
    match sig.recover(message_hash) {
        Ok(recovered_address) => {
            // Use constant-time comparison to prevent timing attacks
            let recovered_bytes = recovered_address.as_bytes();
            let expected_bytes = expected_signer.as_bytes();
            Ok(constant_time_eq(recovered_bytes, expected_bytes))
        },
        Err(e) => Err(ApiError::Validation(format!("Failed to recover signer: {}", e))),
    }
}

/// Secondary signature verification using secp256k1 library
fn verify_signature_secp256k1(
    message: &[u8],
    signature: &str,
    expected_signer: Address,
) -> Result<bool> {
    // Parse signature hex string
    let sig_bytes = hex::decode(signature.strip_prefix("0x").unwrap_or(signature))
        .map_err(|e| ApiError::Validation(format!("Invalid signature hex: {}", e)))?;
    
    if sig_bytes.len() != 65 {
        return Err(ApiError::Validation("Signature must be 65 bytes".to_string()));
    }
    
    // Split signature into r, s, and recovery_id components
    let r = &sig_bytes[0..32];
    let s = &sig_bytes[32..64];
    let recovery_id = sig_bytes[64];
    
    // Create secp256k1 context
    let secp = Secp256k1::new();
    
    // Create recoverable signature
    let recovery_id = RecoveryId::from_i32(recovery_id as i32)
        .map_err(|e| ApiError::Validation(format!("Invalid recovery ID: {}", e)))?;
    
    let mut sig_data = [0u8; 64];
    sig_data[0..32].copy_from_slice(r);
    sig_data[32..64].copy_from_slice(s);
    
    let recoverable_sig = secp256k1::ecdsa::RecoverableSignature::from_compact(&sig_data, recovery_id)
        .map_err(|e| ApiError::Validation(format!("Invalid recoverable signature: {}", e)))?;
    
    // Hash message with Ethereum prefix
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Sha256::new();
    hasher.update(prefixed_message.as_bytes());
    hasher.update(message);
    let message_hash = hasher.finalize();
    
    let message_obj = Secp256k1Message::from_digest_slice(&message_hash)
        .map_err(|e| ApiError::Validation(format!("Invalid message hash: {}", e)))?;
    
    // Recover public key
    let recovered_pubkey = secp.recover_ecdsa(&message_obj, &recoverable_sig)
        .map_err(|e| ApiError::Validation(format!("Failed to recover public key: {}", e)))?;
    
    // Convert public key to Ethereum address
    let pubkey_bytes = recovered_pubkey.serialize_uncompressed();
    let pubkey_hash = keccak256(&pubkey_bytes[1..]);
    let recovered_address = Address::from_slice(&pubkey_hash[12..]);
    
    // Use constant-time comparison
    let recovered_bytes = recovered_address.as_bytes();
    let expected_bytes = expected_signer.as_bytes();
    Ok(constant_time_eq(recovered_bytes, expected_bytes))
}

/// Validate signature verification inputs
fn validate_signature_inputs(
    message: &[u8],
    signature: &str,
    expected_signer: Address,
) -> Result<()> {
    // Check message size limits
    if message.is_empty() {
        return Err(ApiError::Validation("Message cannot be empty".to_string()));
    }
    
    if message.len() > 10_000 {
        return Err(ApiError::Validation("Message too large (max 10KB)".to_string()));
    }
    
    // Validate signature format
    let sig_without_prefix = signature.strip_prefix("0x").unwrap_or(signature);
    if sig_without_prefix.len() != 130 { // 65 bytes * 2 hex chars
        return Err(ApiError::Validation("Signature must be 130 hex characters (65 bytes)".to_string()));
    }
    
    // Validate signature is valid hex
    hex::decode(sig_without_prefix)
        .map_err(|_| ApiError::Validation("Signature must be valid hexadecimal".to_string()))?;
    
    // Validate address is not zero
    if expected_signer.is_zero() {
        return Err(ApiError::Validation("Expected signer cannot be zero address".to_string()));
    }
    
    Ok(())
}

/// Create the message to be signed for solver registration
pub fn create_solver_registration_message(
    solver_address: Address,
    bond_amount: &U256,
    supported_chains: &[u64],
    fee_rate: f64,
) -> Vec<u8> {
    // Create a structured message for signing
    let chains_str = supported_chains
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    
    let message = format!(
        "Orbital Intents Solver Registration\n\
         Solver Address: {:#x}\n\
         Bond Amount: {} wei\n\
         Supported Chains: [{}]\n\
         Fee Rate: {} bps\n\
         \n\
         By signing this message, I confirm that:\n\
         - I am the owner of the solver address\n\
         - I agree to the solver terms and conditions\n\
         - I understand that my bond may be slashed for misbehavior",
        solver_address,
        bond_amount,
        chains_str,
        fee_rate
    );
    
    message.as_bytes().to_vec()
}

/// Verify typed data signature (EIP-712)
pub fn verify_typed_data_signature<T: Encode>(
    domain: &Eip712Domain,
    data: &T,
    signature: &str,
    expected_signer: Address,
) -> Result<bool> {
    // Parse signature
    let sig = Signature::from_str(signature)
        .map_err(|e| ApiError::Validation(format!("Invalid signature format: {}", e)))?;
    
    // Encode the typed data
    let encoded = data.encode_eip712()
        .map_err(|e| ApiError::Validation(format!("Failed to encode typed data: {}", e)))?;
    
    // Create the digest
    let digest = H256::from(keccak256(encoded));
    
    // Recover signer
    match sig.recover(digest) {
        Ok(recovered_address) => Ok(recovered_address == expected_signer),
        Err(e) => Err(ApiError::Validation(format!("Failed to recover signer: {}", e))),
    }
}

/// Create EIP-712 domain for Orbital Intents
pub fn orbital_domain() -> Eip712Domain {
    Eip712Domain {
        name: Some("Orbital Intents".to_string()),
        version: Some("1".to_string()),
        chain_id: None, // Will be set per-chain
        verifying_contract: None,
        salt: None,
    }
}

#[derive(Debug, Clone, Serialize, EthAbiType, EthAbiCodec)]
pub struct SolverRegistrationData {
    pub solver_address: Address,
    pub bond_amount: U256,
    pub supported_chains: Vec<U256>, // Using U256 for chain IDs in EIP-712
    pub fee_rate: U256, // Store as basis points * 100 for precision
    pub nonce: U256,
    pub deadline: U256,
}

impl SolverRegistrationData {
    pub fn new(
        solver_address: Address,
        bond_amount: U256,
        supported_chains: Vec<u64>,
        fee_rate: f64,
        nonce: U256,
        deadline: U256,
    ) -> Self {
        Self {
            solver_address,
            bond_amount,
            supported_chains: supported_chains.iter().map(|&c| U256::from(c)).collect(),
            fee_rate: U256::from((fee_rate * 100.0) as u64), // Convert to basis points * 100
            nonce,
            deadline,
        }
    }
}

/// Generate a cryptographically secure nonce for preventing replay attacks
pub fn generate_secure_nonce() -> U256 {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    U256::from(bytes)
}

/// Verify message freshness using timestamp (prevents replay attacks)
pub fn verify_message_freshness(
    timestamp: u64,
    tolerance_seconds: u64,
) -> Result<()> {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| ApiError::Internal(format!("System time error: {}", e)))?
        .as_secs();
    
    let time_diff = if current_time > timestamp {
        current_time - timestamp
    } else {
        timestamp - current_time
    };
    
    if time_diff > tolerance_seconds {
        return Err(ApiError::Validation(
            format!("Message timestamp too old or too far in future. Diff: {}s", time_diff)
        ));
    }
    
    Ok(())
}

/// Rate limiting for signature verification to prevent DoS attacks
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct SignatureRateLimiter {
    attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_attempts: usize,
    window_duration: Duration,
}

impl SignatureRateLimiter {
    pub fn new(max_attempts: usize, window_seconds: u64) -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window_duration: Duration::from_secs(window_seconds),
        }
    }
    
    pub fn check_rate_limit(&self, identifier: &str) -> Result<()> {
        let mut attempts = self.attempts.lock()
            .map_err(|e| ApiError::Internal(format!("Rate limiter lock error: {}", e)))?;
        
        let now = Instant::now();
        let recent_attempts = attempts.entry(identifier.to_string())
            .or_insert_with(Vec::new);
        
        // Remove old attempts outside the window
        recent_attempts.retain(|&time| now.duration_since(time) < self.window_duration);
        
        // Check if we've exceeded the limit
        if recent_attempts.len() >= self.max_attempts {
            return Err(ApiError::Validation(
                format!("Rate limit exceeded for {}", identifier)
            ));
        }
        
        // Record this attempt
        recent_attempts.push(now);
        
        Ok(())
    }
}

/// Enhanced solver registration message with additional security fields
pub fn create_secure_solver_registration_message(
    solver_address: Address,
    bond_amount: &U256,
    supported_chains: &[u64],
    fee_rate: f64,
    nonce: U256,
    timestamp: u64,
    chain_id: u64,
) -> Vec<u8> {
    let chains_str = supported_chains
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    
    let message = format!(
        "Orbital Intents Solver Registration v2\n\
         Solver Address: {:#x}\n\
         Bond Amount: {} wei\n\
         Supported Chains: [{}]\n\
         Fee Rate: {} bps\n\
         Chain ID: {}\n\
         Nonce: {}\n\
         Timestamp: {}\n\
         \n\
         By signing this message, I confirm that:\n\
         - I am the owner of the solver address\n\
         - I agree to the solver terms and conditions\n\
         - I understand that my bond may be slashed for misbehavior\n\
         - This signature is valid for 5 minutes from timestamp\n\
         - I will not reuse this nonce",
        solver_address,
        bond_amount,
        chains_str,
        fee_rate,
        chain_id,
        nonce,
        timestamp
    );
    
    message.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::signers::{LocalWallet, Signer};
    
    #[tokio::test]
    async fn test_verify_signature() {
        // Create a test wallet
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let address = wallet.address();
        
        // Create a test message
        let message = b"Hello, Orbital Intents!";
        
        // Sign the message
        let signature = wallet.sign_message(message).await.unwrap();
        let sig_str = format!("{}", signature);
        
        // Verify the signature
        let is_valid = verify_signature(message, &sig_str, address).unwrap();
        assert!(is_valid);
        
        // Verify with wrong address should fail
        let wrong_address = Address::random();
        let is_valid = verify_signature(message, &sig_str, wrong_address).unwrap();
        assert!(!is_valid);
    }
    
    #[test]
    fn test_solver_registration_message() {
        let solver_address = Address::random();
        let bond_amount = U256::from(1000000000000000000u64); // 1 ETH
        let supported_chains = vec![1, 137, 42161]; // Ethereum, Polygon, Arbitrum
        let fee_rate = 30.0; // 0.3%
        
        let message = create_solver_registration_message(
            solver_address,
            &bond_amount,
            &supported_chains,
            fee_rate,
        );
        
        let message_str = String::from_utf8(message).unwrap();
        assert!(message_str.contains(&format!("{:#x}", solver_address)));
        assert!(message_str.contains("1000000000000000000 wei"));
        assert!(message_str.contains("1,137,42161"));
        assert!(message_str.contains("30 bps"));
    }
    
    #[tokio::test]
    async fn test_dual_signature_verification() {
        // Create a test wallet
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let address = wallet.address();
        
        // Create a test message
        let message = b"Secure Orbital Intents Test Message";
        
        // Sign the message
        let signature = wallet.sign_message(message).await.unwrap();
        let sig_str = format!("{}", signature);
        
        // Verify the signature using enhanced method
        let is_valid = verify_signature(message, &sig_str, address).unwrap();
        assert!(is_valid);
        
        // Test with invalid signature should fail
        let mut invalid_sig = sig_str.clone();
        invalid_sig.replace_range(..2, "0x");
        let is_valid = verify_signature(message, &invalid_sig, address);
        assert!(is_valid.is_err());
    }
    
    #[test]
    fn test_signature_input_validation() {
        let address = Address::random();
        
        // Test empty message
        let result = validate_signature_inputs(b"", "0x" + &"a".repeat(130), address);
        assert!(result.is_err());
        
        // Test oversized message
        let large_message = vec![0u8; 10001];
        let result = validate_signature_inputs(&large_message, "0x" + &"a".repeat(130), address);
        assert!(result.is_err());
        
        // Test invalid signature length
        let result = validate_signature_inputs(b"test", "0x123", address);
        assert!(result.is_err());
        
        // Test zero address
        let result = validate_signature_inputs(b"test", "0x" + &"a".repeat(130), Address::zero());
        assert!(result.is_err());
        
        // Test valid inputs
        let result = validate_signature_inputs(b"test", "0x" + &"a".repeat(130), address);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_message_freshness() {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Test fresh message
        let result = verify_message_freshness(current_time, 300);
        assert!(result.is_ok());
        
        // Test old message
        let result = verify_message_freshness(current_time - 600, 300);
        assert!(result.is_err());
        
        // Test future message
        let result = verify_message_freshness(current_time + 600, 300);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_rate_limiter() {
        let limiter = SignatureRateLimiter::new(3, 60); // 3 attempts per minute
        let identifier = "test_user";
        
        // First 3 attempts should succeed
        for _ in 0..3 {
            let result = limiter.check_rate_limit(identifier);
            assert!(result.is_ok());
        }
        
        // 4th attempt should fail
        let result = limiter.check_rate_limit(identifier);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_secure_nonce_generation() {
        let nonce1 = generate_secure_nonce();
        let nonce2 = generate_secure_nonce();
        
        // Nonces should be different
        assert_ne!(nonce1, nonce2);
        
        // Nonces should be non-zero
        assert!(!nonce1.is_zero());
        assert!(!nonce2.is_zero());
    }
}