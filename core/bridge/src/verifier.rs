//! Cross-chain proof verification module
//! 
//! This module provides cryptographic verification for cross-chain proofs,
//! including merkle proofs, block headers, and transaction inclusion.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use primitive_types::{H256, U256};
use rlp::{Rlp, RlpStream};
use keccak_hash::keccak;

use crate::{BlockHash, BridgeError, TxHash};

/// Merkle proof for transaction/state inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Leaf value being proven
    pub leaf: Vec<u8>,
    
    /// Merkle proof path (sibling hashes)
    pub siblings: Vec<H256>,
    
    /// Path indices (0 = left, 1 = right)
    pub indices: Vec<bool>,
}

/// Block header structure for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Parent block hash
    pub parent_hash: BlockHash,
    
    /// State root
    pub state_root: H256,
    
    /// Transactions root
    pub transactions_root: H256,
    
    /// Receipts root
    pub receipts_root: H256,
    
    /// Block number
    pub number: u64,
    
    /// Block timestamp
    pub timestamp: u64,
    
    /// Extra data (chain-specific)
    pub extra_data: Vec<u8>,
}

/// Transaction proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionProof {
    /// Transaction data
    pub transaction: Vec<u8>,
    
    /// Transaction index in block
    pub tx_index: u64,
    
    /// Merkle proof for transaction
    pub merkle_proof: MerkleProof,
    
    /// Block header
    pub block_header: BlockHeader,
}

/// State proof data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateProof {
    /// Account address
    pub address: Vec<u8>,
    
    /// Account proof (merkle-patricia proof)
    pub account_proof: Vec<Vec<u8>>,
    
    /// Storage key (if proving storage)
    pub storage_key: Option<H256>,
    
    /// Storage proof (if proving storage)
    pub storage_proof: Option<Vec<Vec<u8>>>,
    
    /// State root
    pub state_root: H256,
}

/// Proof verifier for different proof types
pub struct ProofVerifier;

impl ProofVerifier {
    /// Verify a merkle proof
    pub fn verify_merkle_proof(
        proof: &MerkleProof,
        root: &H256,
    ) -> Result<bool, BridgeError> {
        if proof.siblings.len() != proof.indices.len() {
            return Err(BridgeError::ProofValidationFailed(
                "Invalid proof length".to_string()
            ));
        }
        
        // Start with leaf hash
        let mut current_hash = Self::hash_leaf(&proof.leaf);
        
        // Traverse up the tree
        for (i, sibling) in proof.siblings.iter().enumerate() {
            let is_right = proof.indices[i];
            
            current_hash = if is_right {
                Self::hash_pair(sibling, &current_hash)
            } else {
                Self::hash_pair(&current_hash, sibling)
            };
        }
        
        Ok(&current_hash == root)
    }
    
    /// Verify transaction inclusion in a block
    pub fn verify_transaction_inclusion(
        proof: &TransactionProof,
    ) -> Result<bool, BridgeError> {
        // Verify the transaction is included in the transactions root
        let tx_hash = keccak(&proof.transaction);
        
        let mut leaf_data = Vec::new();
        leaf_data.extend_from_slice(&proof.tx_index.to_le_bytes());
        leaf_data.extend_from_slice(tx_hash.as_bytes());
        
        let merkle_proof = MerkleProof {
            leaf: leaf_data,
            siblings: proof.merkle_proof.siblings.clone(),
            indices: proof.merkle_proof.indices.clone(),
        };
        
        Self::verify_merkle_proof(
            &merkle_proof,
            &proof.block_header.transactions_root,
        )
    }
    
    /// Verify block header validity
    pub fn verify_block_header(
        header: &BlockHeader,
        parent_hash: &BlockHash,
    ) -> Result<bool, BridgeError> {
        // Verify parent hash matches
        if &header.parent_hash != parent_hash {
            return Ok(false);
        }
        
        // Additional checks could include:
        // - Timestamp validation
        // - Gas limit checks
        // - Difficulty/PoS checks
        
        Ok(true)
    }
    
    /// Verify state proof (Merkle-Patricia proof)
    pub fn verify_state_proof(
        proof: &StateProof,
    ) -> Result<Vec<u8>, BridgeError> {
        // This is a simplified version
        // Real implementation would need full MPT verification
        
        if proof.account_proof.is_empty() {
            return Err(BridgeError::ProofValidationFailed(
                "Empty account proof".to_string()
            ));
        }
        
        // Verify account exists in state trie
        let account_data = Self::verify_mpt_proof(
            &proof.address,
            &proof.account_proof,
            &proof.state_root,
        )?;
        
        // If storage proof provided, verify storage value
        if let (Some(storage_key), Some(storage_proof)) = 
            (&proof.storage_key, &proof.storage_proof) {
            
            // Extract storage root from account data
            // This would parse RLP-encoded account data
            let _storage_value = Self::verify_mpt_proof(
                storage_key.as_bytes(),
                storage_proof,
                &H256::zero(), // Would be actual storage root
            )?;
        }
        
        Ok(account_data)
    }
    
    /// Verify receipt inclusion and extract logs
    pub fn verify_receipt_proof(
        receipt_data: &[u8],
        receipt_index: u64,
        receipts_root: &H256,
        merkle_proof: &MerkleProof,
    ) -> Result<bool, BridgeError> {
        // Create leaf data from receipt
        let receipt_hash = keccak(receipt_data);
        
        let mut leaf_data = Vec::new();
        leaf_data.extend_from_slice(&receipt_index.to_le_bytes());
        leaf_data.extend_from_slice(receipt_hash.as_bytes());
        
        let proof = MerkleProof {
            leaf: leaf_data,
            siblings: merkle_proof.siblings.clone(),
            indices: merkle_proof.indices.clone(),
        };
        
        Self::verify_merkle_proof(&proof, receipts_root)
    }
    
    /// Hash a leaf node
    fn hash_leaf(data: &[u8]) -> H256 {
        H256::from_slice(keccak(data).as_bytes())
    }
    
    /// Hash two nodes together
    fn hash_pair(left: &H256, right: &H256) -> H256 {
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(left.as_bytes());
        data.extend_from_slice(right.as_bytes());
        
        H256::from_slice(keccak(&data).as_bytes())
    }
    
    /// Verify Merkle-Patricia Trie proof (simplified)
    fn verify_mpt_proof(
        key: &[u8],
        proof: &[Vec<u8>],
        root: &H256,
    ) -> Result<Vec<u8>, BridgeError> {
        // This is a simplified implementation
        // Real MPT verification is complex
        
        if proof.is_empty() {
            return Err(BridgeError::ProofValidationFailed(
                "Empty MPT proof".to_string()
            ));
        }
        
        // In a real implementation:
        // 1. Start from root
        // 2. Follow the path defined by the key
        // 3. Verify each node hash matches
        // 4. Return the value at the leaf
        
        // For now, return dummy data
        Ok(vec![0u8; 32])
    }
}

/// Light client for block header verification
pub struct LightClient {
    /// Trusted block headers
    trusted_headers: Vec<BlockHeader>,
    
    /// Maximum headers to store
    max_headers: usize,
}

impl LightClient {
    pub fn new(genesis: BlockHeader) -> Self {
        Self {
            trusted_headers: vec![genesis],
            max_headers: 1000,
        }
    }
    
    /// Add a new verified header
    pub fn add_header(&mut self, header: BlockHeader) -> Result<(), BridgeError> {
        // Verify against latest trusted header
        if let Some(parent) = self.trusted_headers.last() {
            let parent_hash = Self::hash_header(parent);
            
            if !ProofVerifier::verify_block_header(&header, &parent_hash)? {
                return Err(BridgeError::VerificationFailed(
                    "Invalid block header".to_string()
                ));
            }
        }
        
        self.trusted_headers.push(header);
        
        // Prune old headers
        if self.trusted_headers.len() > self.max_headers {
            self.trusted_headers.remove(0);
        }
        
        Ok(())
    }
    
    /// Get a trusted header by height
    pub fn get_header(&self, height: u64) -> Option<&BlockHeader> {
        self.trusted_headers.iter()
            .find(|h| h.number == height)
    }
    
    /// Verify a proof against trusted headers
    pub fn verify_against_trusted(
        &self,
        block_height: u64,
        state_root: &H256,
    ) -> Result<bool, BridgeError> {
        if let Some(header) = self.get_header(block_height) {
            Ok(&header.state_root == state_root)
        } else {
            Err(BridgeError::VerificationFailed(
                format!("No trusted header at height {}", block_height)
            ))
        }
    }
    
    /// Hash a block header
    fn hash_header(header: &BlockHeader) -> BlockHash {
        let mut stream = RlpStream::new();
        
        stream.append(&header.parent_hash.as_ref());
        stream.append(&header.state_root.as_bytes());
        stream.append(&header.transactions_root.as_bytes());
        stream.append(&header.receipts_root.as_bytes());
        stream.append(&header.number);
        stream.append(&header.timestamp);
        stream.append(&header.extra_data);
        
        let rlp_encoded = stream.out();
        let hash = keccak(&rlp_encoded);
        
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }
}

/// Signature verification for validator-based bridges
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verify a validator signature
    pub fn verify_signature(
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Result<bool, BridgeError> {
        // This would use actual cryptographic verification
        // For now, simplified check
        
        if signature.len() != 65 || public_key.len() != 33 {
            return Err(BridgeError::VerificationFailed(
                "Invalid signature format".to_string()
            ));
        }
        
        // In production:
        // 1. Recover signer from signature
        // 2. Verify it matches the public key
        // 3. Check signature is valid for message
        
        Ok(true)
    }
    
    /// Verify a threshold of signatures
    pub fn verify_threshold_signatures(
        message: &[u8],
        signatures: &[Vec<u8>],
        validators: &[Vec<u8>],
        threshold: usize,
    ) -> Result<bool, BridgeError> {
        if signatures.len() < threshold {
            return Err(BridgeError::VerificationFailed(
                format!("Insufficient signatures: {} < {}", signatures.len(), threshold)
            ));
        }
        
        let mut valid_count = 0;
        
        for (sig, validator) in signatures.iter().zip(validators.iter()) {
            if Self::verify_signature(message, sig, validator)? {
                valid_count += 1;
            }
        }
        
        Ok(valid_count >= threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle_proof_verification() {
        // Create a simple merkle tree
        let leaf = b"test_data".to_vec();
        let sibling = H256::from_low_u64_be(12345);
        let root = ProofVerifier::hash_pair(
            &ProofVerifier::hash_leaf(&leaf),
            &sibling,
        );
        
        let proof = MerkleProof {
            leaf,
            siblings: vec![sibling],
            indices: vec![false], // leaf is on left
        };
        
        assert!(ProofVerifier::verify_merkle_proof(&proof, &root).unwrap());
    }
    
    #[test]
    fn test_block_header_verification() {
        let parent = BlockHeader {
            parent_hash: [0u8; 32],
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            number: 100,
            timestamp: 1234567890,
            extra_data: vec![],
        };
        
        let parent_hash = LightClient::hash_header(&parent);
        
        let child = BlockHeader {
            parent_hash,
            state_root: H256::from_low_u64_be(1),
            transactions_root: H256::from_low_u64_be(2),
            receipts_root: H256::from_low_u64_be(3),
            number: 101,
            timestamp: 1234567900,
            extra_data: vec![],
        };
        
        assert!(ProofVerifier::verify_block_header(&child, &parent_hash).unwrap());
    }
    
    #[test]
    fn test_light_client() {
        let genesis = BlockHeader {
            parent_hash: [0u8; 32],
            state_root: H256::zero(),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            number: 0,
            timestamp: 1234567890,
            extra_data: vec![],
        };
        
        let mut client = LightClient::new(genesis.clone());
        
        let next_header = BlockHeader {
            parent_hash: LightClient::hash_header(&genesis),
            state_root: H256::from_low_u64_be(1),
            transactions_root: H256::zero(),
            receipts_root: H256::zero(),
            number: 1,
            timestamp: 1234567900,
            extra_data: vec![],
        };
        
        assert!(client.add_header(next_header).is_ok());
        assert_eq!(client.trusted_headers.len(), 2);
    }
}