//! Example of using the cross-chain bridge for token transfers

use intents_bridge::{
    BridgeManager, BridgeProtocol, CrossChainMessage, 
    protocols::{LayerZeroBridge, AxelarBridge, WormholeBridge},
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create bridge manager with LayerZero as default
    let mut bridge_manager = BridgeManager::new(BridgeProtocol::LayerZero);
    
    // Register bridge implementations
    bridge_manager.register_bridge(Box::new(LayerZeroBridge::new()));
    bridge_manager.register_bridge(Box::new(AxelarBridge::new()));
    bridge_manager.register_bridge(Box::new(WormholeBridge::new()));
    
    // Create a cross-chain message for USDC transfer
    let message = CrossChainMessage {
        source_chain: 1,      // Ethereum
        dest_chain: 137,      // Polygon
        nonce: 1,
        sender: vec![0x11; 20],
        receiver: vec![0x22; 20],
        payload: create_transfer_payload(1000_000_000), // 1000 USDC
        timestamp: 1234567890,
        metadata: HashMap::new(),
    };
    
    // Find best bridge for this route
    if let Some(bridge) = bridge_manager.find_best_bridge(1, 137).await {
        println!("Using bridge: {:?}", bridge.protocol());
        
        // Estimate fees
        let fee = bridge.estimate_fees(1, 137, message.payload.len()).await?;
        println!("Estimated fee: {} wei", fee);
        
        // Send message
        match bridge.send_message(message).await {
            Ok(receipt) => {
                println!("Message sent successfully!");
                println!("Message ID: 0x{}", hex::encode(receipt.message_id));
                println!("Source TX: 0x{}", hex::encode(receipt.source_tx));
                println!("Status: {:?}", receipt.status);
            }
            Err(e) => {
                eprintln!("Failed to send message: {}", e);
            }
        }
    } else {
        eprintln!("No compatible bridge found for route");
    }
    
    Ok(())
}

fn create_transfer_payload(amount: u64) -> Vec<u8> {
    // Simple payload structure: [function_selector(4) + amount(32)]
    let mut payload = vec![0xa9, 0x05, 0x9c, 0xbb]; // transfer selector
    payload.extend_from_slice(&amount.to_be_bytes());
    payload.resize(36, 0); // Pad to 32 bytes for amount
    payload
}