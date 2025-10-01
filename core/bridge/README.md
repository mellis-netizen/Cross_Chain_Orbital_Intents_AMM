# Cross-Chain Bridge Module

The bridge module provides a unified interface for cross-chain communication, supporting multiple bridge protocols and verification mechanisms.

## Features

- **Multiple Bridge Protocols**:
  - LayerZero - Ultra-light node messaging
  - Axelar - General message passing
  - Wormhole - Guardian-based bridging
  - Optimistic Rollups - L1/L2 native bridges

- **Proof Verification**:
  - Merkle proof verification
  - Block header validation
  - Transaction inclusion proofs
  - State proof verification

- **Bridge Management**:
  - Protocol abstraction
  - Route optimization
  - Fee estimation
  - Message tracking

## Architecture

```
bridge/
├── src/
│   ├── lib.rs       # Core traits and types
│   ├── protocols.rs # Bridge implementations
│   └── verifier.rs  # Proof verification
├── tests/           # Integration tests
└── examples/        # Usage examples
```

## Usage

### Basic Cross-Chain Transfer

```rust
use intents_bridge::{BridgeManager, BridgeProtocol, CrossChainMessage};

// Create bridge manager
let mut manager = BridgeManager::new(BridgeProtocol::LayerZero);

// Register bridges
manager.register_bridge(Box::new(LayerZeroBridge::new()));

// Send cross-chain message
let message = CrossChainMessage {
    source_chain: 1,    // Ethereum
    dest_chain: 137,    // Polygon
    // ... message details
};

if let Some(bridge) = manager.find_best_bridge(1, 137).await {
    let receipt = bridge.send_message(message).await?;
    println!("Message sent: {:?}", receipt);
}
```

### Proof Verification

```rust
use intents_bridge::verifier::{ProofVerifier, MerkleProof};

let proof = MerkleProof {
    leaf: transaction_data,
    siblings: vec![sibling_hashes],
    indices: vec![path_indices],
};

let is_valid = ProofVerifier::verify_merkle_proof(&proof, &root)?;
```

## Bridge Implementations

### LayerZero
- Supports ultra-light node architecture
- Efficient for frequent small messages
- Oracle and relayer based verification

### Axelar
- General message passing protocol
- Validator-based consensus
- Supports complex cross-chain calls

### Wormhole
- Guardian network validation
- VAA (Verified Action Approval) based
- Low fees, high security

### Optimistic Rollups
- Native L1/L2 bridges
- Challenge period for withdrawals
- Most secure for L2 interactions

## Integration with Intent Engine

The bridge module integrates with the intent engine to enable cross-chain intent execution:

```rust
// In intent executor
if intent.is_cross_chain() {
    let bridge = bridge_manager.find_best_bridge(
        intent.source_chain,
        intent.dest_chain
    ).await?;
    
    let message = intent.to_cross_chain_message();
    bridge.send_message(message).await?;
}
```

## Testing

Run the test suite:

```bash
cargo test -p intents-bridge
```

Run examples:

```bash
cargo run -p intents-bridge --example cross_chain_transfer
```