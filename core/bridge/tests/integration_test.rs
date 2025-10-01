//! Integration tests for the cross-chain bridge module

use intents_bridge::{
    Bridge, BridgeManager, BridgeProtocol, CrossChainMessage, CrossChainProof,
    protocols::{LayerZeroBridge, AxelarBridge, WormholeBridge, OptimisticRollupBridge},
    verifier::{ProofVerifier, MerkleProof, BlockHeader, TransactionProof},
};
use std::collections::HashMap;
use primitive_types::H256;

#[tokio::test]
async fn test_bridge_manager_registration() {
    let mut manager = BridgeManager::new(BridgeProtocol::LayerZero);
    
    // Register multiple bridges
    manager.register_bridge(Box::new(LayerZeroBridge::new()));
    manager.register_bridge(Box::new(AxelarBridge::new()));
    manager.register_bridge(Box::new(WormholeBridge::new()));
    
    // Check bridges are registered
    assert!(manager.get_bridge(&BridgeProtocol::LayerZero).is_some());
    assert!(manager.get_bridge(&BridgeProtocol::Axelar).is_some());
    assert!(manager.get_bridge(&BridgeProtocol::Wormhole).is_some());
}

#[tokio::test]
async fn test_find_best_bridge() {
    let mut manager = BridgeManager::new(BridgeProtocol::LayerZero);
    
    // Register bridges
    manager.register_bridge(Box::new(LayerZeroBridge::new()));
    manager.register_bridge(Box::new(AxelarBridge::new()));
    
    // Find bridge for Ethereum -> Polygon
    let bridge = manager.find_best_bridge(1, 137).await;
    assert!(bridge.is_some());
    
    // Find bridge for unsupported route
    let bridge = manager.find_best_bridge(1, 999999).await;
    assert!(bridge.is_none());
}

#[tokio::test]
async fn test_cross_chain_message_flow() {
    let bridge = LayerZeroBridge::new();
    
    let message = CrossChainMessage {
        source_chain: 1,    // Ethereum
        dest_chain: 137,    // Polygon
        nonce: 100,
        sender: vec![0xaa; 20],
        receiver: vec![0xbb; 20],
        payload: b"Hello Cross-Chain World!".to_vec(),
        timestamp: 1234567890,
        metadata: HashMap::new(),
    };
    
    // Send message
    let receipt = bridge.send_message(message.clone()).await.unwrap();
    assert_eq!(receipt.message_id, intents_bridge::hash_message(&message));
    
    // Estimate fees
    let fee = bridge.estimate_fees(1, 137, message.payload.len()).await.unwrap();
    assert!(fee > 0);
}

#[tokio::test]
async fn test_merkle_proof_verification() {
    // Create a simple merkle proof
    let leaf_data = b"transaction_data".to_vec();
    let leaf_hash = H256::from_slice(&sha2::Sha256::digest(&leaf_data).as_slice());
    let sibling = H256::from_low_u64_be(999);
    
    // Calculate root
    let mut root_data = Vec::new();
    root_data.extend_from_slice(leaf_hash.as_bytes());
    root_data.extend_from_slice(sibling.as_bytes());
    let root = H256::from_slice(&sha2::Sha256::digest(&root_data).as_slice());
    
    let proof = MerkleProof {
        leaf: leaf_data,
        siblings: vec![sibling],
        indices: vec![false], // leaf is on left
    };
    
    // Verify proof
    assert!(ProofVerifier::verify_merkle_proof(&proof, &root).unwrap());
}

#[tokio::test]
async fn test_optimistic_rollup_bridge() {
    let bridge = OptimisticRollupBridge::new(1, 10); // Ethereum -> Optimism
    
    // Test deposit (L1 -> L2)
    let deposit_message = CrossChainMessage {
        source_chain: 1,
        dest_chain: 10,
        nonce: 1,
        sender: vec![0x11; 20],
        receiver: vec![0x22; 20],
        payload: vec![0x33; 100],
        timestamp: 1234567890,
        metadata: HashMap::new(),
    };
    
    let receipt = bridge.send_message(deposit_message).await.unwrap();
    assert!(receipt.message_id != [0u8; 32]);
    
    // Test withdrawal (L2 -> L1)
    let withdrawal_message = CrossChainMessage {
        source_chain: 10,
        dest_chain: 1,
        nonce: 2,
        sender: vec![0x22; 20],
        receiver: vec![0x11; 20],
        payload: vec![0x44; 100],
        timestamp: 1234567891,
        metadata: HashMap::new(),
    };
    
    let receipt = bridge.send_message(withdrawal_message).await.unwrap();
    assert!(receipt.message_id != [0u8; 32]);
    
    // Test fee estimation
    let deposit_fee = bridge.estimate_fees(1, 10, 100).await.unwrap();
    let withdrawal_fee = bridge.estimate_fees(10, 1, 100).await.unwrap();
    
    // Withdrawals should be more expensive due to L1 proof submission
    assert!(withdrawal_fee > deposit_fee);
}

#[tokio::test]
async fn test_protocol_specific_features() {
    // Test Axelar chain name mapping
    let axelar = AxelarBridge::new();
    assert_eq!(axelar.supported_chains(), vec![1, 137, 42161]);
    
    // Test Wormhole VAA verification
    let wormhole = WormholeBridge::new();
    let message = CrossChainMessage {
        source_chain: 1,
        dest_chain: 137,
        nonce: 1,
        sender: vec![0x01; 20],
        receiver: vec![0x02; 20],
        payload: vec![],
        timestamp: 1234567890,
        metadata: HashMap::new(),
    };
    
    // Proof without VAA should fail
    let proof = CrossChainProof {
        block_height: 100,
        block_hash: [0u8; 32],
        tx_hash: [0u8; 32],
        merkle_proof: vec![],
        proof_data: HashMap::new(),
    };
    
    let result = wormhole.verify_message(&message, &proof).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_bridge_operations() {
    use tokio::task;
    
    let mut manager = BridgeManager::new(BridgeProtocol::LayerZero);
    manager.register_bridge(Box::new(LayerZeroBridge::new()));
    manager.register_bridge(Box::new(AxelarBridge::new()));
    manager.register_bridge(Box::new(WormholeBridge::new()));
    
    // Simulate concurrent cross-chain transfers
    let mut handles = vec![];
    
    for i in 0..5 {
        let message = CrossChainMessage {
            source_chain: 1,
            dest_chain: 137,
            nonce: i as u64,
            sender: vec![i as u8; 20],
            receiver: vec![(i + 1) as u8; 20],
            payload: format!("Transfer {}", i).as_bytes().to_vec(),
            timestamp: 1234567890 + i as u64,
            metadata: HashMap::new(),
        };
        
        handles.push(task::spawn(async move {
            let bridge = LayerZeroBridge::new();
            bridge.send_message(message).await
        }));
    }
    
    // Wait for all transfers
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}