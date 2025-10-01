//! Cross-chain bridge module for the Rust intents system
//! 
//! This module provides abstractions and implementations for cross-chain
//! communication, message passing, and state verification across different
//! blockchain networks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

pub mod protocols;
pub mod verifier;

/// Chain identifier type
pub type ChainId = u64;

/// Transaction hash type (32 bytes)
pub type TxHash = [u8; 32];

/// Block hash type (32 bytes)
pub type BlockHash = [u8; 32];

/// Bridge error types
#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(ChainId),
    
    #[error("Message verification failed: {0}")]
    VerificationFailed(String),
    
    #[error("Protocol not supported: {0}")]
    ProtocolNotSupported(String),
    
    #[error("State sync failed: {0}")]
    StateSyncFailed(String),
    
    #[error("Proof validation failed: {0}")]
    ProofValidationFailed(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Cross-chain message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    /// Source chain ID
    pub source_chain: ChainId,
    
    /// Destination chain ID
    pub dest_chain: ChainId,
    
    /// Message nonce for replay protection
    pub nonce: u64,
    
    /// Sender address on source chain
    pub sender: Vec<u8>,
    
    /// Receiver address on destination chain
    pub receiver: Vec<u8>,
    
    /// Message payload
    pub payload: Vec<u8>,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Protocol-specific metadata
    pub metadata: HashMap<String, Vec<u8>>,
}

/// Cross-chain proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainProof {
    /// Block height
    pub block_height: u64,
    
    /// Block hash
    pub block_hash: BlockHash,
    
    /// Transaction hash
    pub tx_hash: TxHash,
    
    /// Merkle proof
    pub merkle_proof: Vec<Vec<u8>>,
    
    /// Additional protocol-specific proof data
    pub proof_data: HashMap<String, Vec<u8>>,
}

/// Message receipt after successful cross-chain transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    /// Message ID
    pub message_id: [u8; 32],
    
    /// Source transaction hash
    pub source_tx: TxHash,
    
    /// Destination transaction hash (if executed)
    pub dest_tx: Option<TxHash>,
    
    /// Execution status
    pub status: MessageStatus,
    
    /// Timestamp
    pub timestamp: u64,
}

/// Message execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    Pending,
    Validated,
    Executed,
    Failed(String),
}

/// Bridge protocol types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BridgeProtocol {
    LayerZero,
    Axelar,
    Wormhole,
    OptimisticRollup,
    Custom(String),
}

/// State verification result
#[derive(Debug, Clone)]
pub struct StateVerification {
    /// Whether the state is valid
    pub is_valid: bool,
    
    /// Verified block height
    pub block_height: u64,
    
    /// State root
    pub state_root: [u8; 32],
    
    /// Additional verification data
    pub metadata: HashMap<String, Vec<u8>>,
}

/// Core bridge trait that all bridge implementations must implement
#[async_trait]
pub trait Bridge: Send + Sync {
    /// Get the bridge protocol type
    fn protocol(&self) -> BridgeProtocol;
    
    /// Get supported chains
    fn supported_chains(&self) -> Vec<ChainId>;
    
    /// Send a cross-chain message
    async fn send_message(
        &self,
        message: CrossChainMessage,
    ) -> Result<MessageReceipt, BridgeError>;
    
    /// Verify a cross-chain message with proof
    async fn verify_message(
        &self,
        message: &CrossChainMessage,
        proof: &CrossChainProof,
    ) -> Result<bool, BridgeError>;
    
    /// Get message status
    async fn get_message_status(
        &self,
        message_id: [u8; 32],
    ) -> Result<MessageStatus, BridgeError>;
    
    /// Verify state across chains
    async fn verify_state(
        &self,
        chain_id: ChainId,
        block_height: u64,
        state_data: Vec<u8>,
    ) -> Result<StateVerification, BridgeError>;
    
    /// Estimate fees for cross-chain message
    async fn estimate_fees(
        &self,
        source_chain: ChainId,
        dest_chain: ChainId,
        payload_size: usize,
    ) -> Result<u64, BridgeError>;
}

/// Bridge manager for handling multiple bridge protocols
pub struct BridgeManager {
    bridges: HashMap<BridgeProtocol, Box<dyn Bridge>>,
    default_protocol: BridgeProtocol,
}

impl BridgeManager {
    /// Create a new bridge manager
    pub fn new(default_protocol: BridgeProtocol) -> Self {
        Self {
            bridges: HashMap::new(),
            default_protocol,
        }
    }
    
    /// Register a bridge implementation
    pub fn register_bridge(&mut self, bridge: Box<dyn Bridge>) {
        let protocol = bridge.protocol();
        self.bridges.insert(protocol, bridge);
    }
    
    /// Get a bridge by protocol
    pub fn get_bridge(&self, protocol: &BridgeProtocol) -> Option<&Box<dyn Bridge>> {
        self.bridges.get(protocol)
    }
    
    /// Get the default bridge
    pub fn default_bridge(&self) -> Option<&Box<dyn Bridge>> {
        self.bridges.get(&self.default_protocol)
    }
    
    /// Find best bridge for a route
    pub async fn find_best_bridge(
        &self,
        source_chain: ChainId,
        dest_chain: ChainId,
    ) -> Option<&Box<dyn Bridge>> {
        // Find bridges that support both chains
        let mut compatible_bridges = Vec::new();
        
        for (protocol, bridge) in &self.bridges {
            let supported = bridge.supported_chains();
            if supported.contains(&source_chain) && supported.contains(&dest_chain) {
                compatible_bridges.push((protocol, bridge));
            }
        }
        
        // For now, return the first compatible bridge
        // In production, this would consider fees, speed, reliability, etc.
        compatible_bridges.first().map(|(_, bridge)| bridge)
    }
}

/// Message hasher for generating message IDs
pub fn hash_message(message: &CrossChainMessage) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(message.source_chain.to_le_bytes());
    hasher.update(message.dest_chain.to_le_bytes());
    hasher.update(message.nonce.to_le_bytes());
    hasher.update(&message.sender);
    hasher.update(&message.receiver);
    hasher.update(&message.payload);
    hasher.update(message.timestamp.to_le_bytes());
    
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_message_hashing() {
        let message = CrossChainMessage {
            source_chain: 1,
            dest_chain: 137,
            nonce: 100,
            sender: vec![0x01; 20],
            receiver: vec![0x02; 20],
            payload: vec![0x03; 32],
            timestamp: 1234567890,
            metadata: HashMap::new(),
        };
        
        let hash1 = hash_message(&message);
        let hash2 = hash_message(&message);
        
        // Hash should be deterministic
        assert_eq!(hash1, hash2);
        
        // Hash should change with different data
        let mut message2 = message.clone();
        message2.nonce = 101;
        let hash3 = hash_message(&message2);
        
        assert_ne!(hash1, hash3);
    }
}