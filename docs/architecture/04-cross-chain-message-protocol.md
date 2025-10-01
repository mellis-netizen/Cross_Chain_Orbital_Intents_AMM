# Cross-Chain Message Protocol

## Overview
The Cross-Chain Message Protocol enables reliable, secure, and efficient communication between different blockchain networks for intent execution, state synchronization, and asset transfers.

## Architecture

### Core Components

#### 1. Message Types

```rust
use ethers::types::{Address, U256, H256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Intent execution request
    IntentExecution(IntentExecutionMessage),

    /// State synchronization
    StateSync(StateSyncMessage),

    /// Asset transfer
    AssetTransfer(AssetTransferMessage),

    /// Proof verification
    ProofVerification(ProofVerificationMessage),

    /// Acknowledgment
    Acknowledgment(AcknowledgmentMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    /// Unique message ID
    pub message_id: H256,

    /// Source chain ID
    pub source_chain: u64,

    /// Destination chain ID
    pub dest_chain: u64,

    /// Sender address (source chain format)
    pub sender: Address,

    /// Receiver address (destination chain format)
    pub receiver: Address,

    /// Nonce for replay protection
    pub nonce: u64,

    /// Message type and payload
    pub message_type: MessageType,

    /// Timestamp
    pub timestamp: u64,

    /// Expiry timestamp
    pub expiry: u64,

    /// Gas limit for execution
    pub gas_limit: U256,

    /// Message priority (0=low, 255=critical)
    pub priority: u8,

    /// Protocol-specific metadata
    pub metadata: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentExecutionMessage {
    pub intent_id: H256,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub min_amount_out: U256,
    pub solver: Address,
    pub execution_params: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSyncMessage {
    pub pool_id: H256,
    pub reserve0: U256,
    pub reserve1: U256,
    pub virtual_reserve0: U256,
    pub virtual_reserve1: U256,
    pub state_root: H256,
    pub block_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransferMessage {
    pub transfer_id: H256,
    pub token: Address,
    pub amount: U256,
    pub recipient: Address,
    pub lock_period: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationMessage {
    pub proof_type: ProofType,
    pub proof_data: Vec<u8>,
    pub subject: H256,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProofType {
    Transaction,
    State,
    Receipt,
    Merkle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcknowledgmentMessage {
    pub original_message_id: H256,
    pub status: MessageStatus,
    pub result: Vec<u8>,
    pub gas_used: U256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Validated,
    Executing,
    Executed,
    Failed,
    Expired,
}
```

#### 2. Message Router

```rust
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("Chain not supported: {0}")]
    ChainNotSupported(u64),

    #[error("Route not found for {0} -> {1}")]
    RouteNotFound(u64, u64),

    #[error("Message delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

pub type RouterResult<T> = std::result::Result<T, RouterError>;

/// Route configuration for message delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Source chain
    pub source: u64,

    /// Destination chain
    pub dest: u64,

    /// Bridge protocol to use
    pub protocol: BridgeProtocol,

    /// Estimated delivery time (seconds)
    pub estimated_time: u64,

    /// Base fee in source chain native token
    pub base_fee: U256,

    /// Per-byte fee
    pub byte_fee: U256,

    /// Maximum message size
    pub max_size: usize,

    /// Reliability score (0-10000)
    pub reliability: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeProtocol {
    LayerZero,
    Axelar,
    Wormhole,
    Hyperlane,
    Chainlink CCIP,
    Custom,
}

pub struct MessageRouter {
    /// Available routes by source -> dest
    routes: HashMap<(u64, u64), Vec<Route>>,

    /// Bridge adapters
    adapters: HashMap<BridgeProtocol, Box<dyn BridgeAdapter>>,

    /// Message queue
    pending_messages: Arc<RwLock<HashMap<H256, CrossChainMessage>>>,

    /// Delivery receipts
    receipts: Arc<RwLock<HashMap<H256, MessageReceipt>>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            adapters: HashMap::new(),
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            receipts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a route between chains
    pub fn register_route(&mut self, route: Route) {
        let key = (route.source, route.dest);
        self.routes.entry(key).or_insert_with(Vec::new).push(route);
    }

    /// Register a bridge adapter
    pub fn register_adapter(
        &mut self,
        protocol: BridgeProtocol,
        adapter: Box<dyn BridgeAdapter>,
    ) {
        self.adapters.insert(protocol, adapter);
    }

    /// Send a cross-chain message
    pub async fn send_message(
        &self,
        message: CrossChainMessage,
    ) -> RouterResult<MessageReceipt> {
        // Validate message
        self.validate_message(&message)?;

        // Find best route
        let route = self
            .find_best_route(message.source_chain, message.dest_chain, &message)?;

        // Get adapter for the route
        let adapter = self
            .adapters
            .get(&route.protocol)
            .ok_or(RouterError::ChainNotSupported(message.dest_chain))?;

        // Store in pending
        {
            let mut pending = self.pending_messages.write().await;
            pending.insert(message.message_id, message.clone());
        }

        // Send via adapter
        let receipt = adapter.send_message(message, route).await?;

        // Store receipt
        {
            let mut receipts = self.receipts.write().await;
            receipts.insert(receipt.message_id, receipt.clone());
        }

        Ok(receipt)
    }

    /// Check message status
    pub async fn get_message_status(
        &self,
        message_id: H256,
    ) -> RouterResult<MessageStatus> {
        let receipts = self.receipts.read().await;

        receipts
            .get(&message_id)
            .map(|r| r.status)
            .ok_or(RouterError::InvalidMessage(
                "Message not found".to_string()
            ))
    }

    /// Wait for message delivery
    pub async fn wait_for_delivery(
        &self,
        message_id: H256,
        timeout: u64,
    ) -> RouterResult<MessageReceipt> {
        let start = std::time::Instant::now();

        loop {
            {
                let receipts = self.receipts.read().await;
                if let Some(receipt) = receipts.get(&message_id) {
                    match receipt.status {
                        MessageStatus::Executed => return Ok(receipt.clone()),
                        MessageStatus::Failed | MessageStatus::Expired => {
                            return Err(RouterError::DeliveryFailed(
                                format!("Message delivery failed with status: {:?}", receipt.status)
                            ));
                        }
                        _ => {}
                    }
                }
            }

            if start.elapsed().as_secs() > timeout {
                return Err(RouterError::DeliveryFailed(
                    "Timeout waiting for delivery".to_string()
                ));
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    /// Find best route for message
    fn find_best_route(
        &self,
        source: u64,
        dest: u64,
        message: &CrossChainMessage,
    ) -> RouterResult<Route> {
        let routes = self
            .routes
            .get(&(source, dest))
            .ok_or(RouterError::RouteNotFound(source, dest))?;

        // Score routes based on criteria
        let mut scored: Vec<_> = routes
            .iter()
            .map(|r| {
                let score = self.score_route(r, message);
                (r.clone(), score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scored
            .first()
            .map(|(r, _)| r.clone())
            .ok_or(RouterError::RouteNotFound(source, dest))
    }

    /// Score a route for a message
    fn score_route(&self, route: &Route, message: &CrossChainMessage) -> f64 {
        // Factors: reliability, speed, cost, priority

        let reliability_score = route.reliability as f64 / 10000.0;

        let speed_score = if message.priority > 200 {
            // High priority: prefer speed
            1.0 / (route.estimated_time as f64 + 1.0)
        } else {
            0.5 / (route.estimated_time as f64 + 1.0)
        };

        let message_size = bincode::serialize(&message).unwrap().len();
        let cost = route.base_fee.as_u128() as f64
            + route.byte_fee.as_u128() as f64 * message_size as f64;
        let cost_score = 1.0 / (cost + 1.0);

        // Weighted combination
        reliability_score * 0.4 + speed_score * 0.3 + cost_score * 0.3
    }

    /// Validate message
    fn validate_message(&self, message: &CrossChainMessage) -> RouterResult<()> {
        // Check expiry
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if message.expiry < now {
            return Err(RouterError::InvalidMessage(
                "Message already expired".to_string()
            ));
        }

        // Check chains are different
        if message.source_chain == message.dest_chain {
            return Err(RouterError::InvalidMessage(
                "Source and dest chains must be different".to_string()
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    pub message_id: H256,
    pub source_tx: Option<H256>,
    pub dest_tx: Option<H256>,
    pub status: MessageStatus,
    pub timestamp: u64,
    pub gas_used: U256,
    pub fee_paid: U256,
}
```

#### 3. Bridge Adapter Interface

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait BridgeAdapter: Send + Sync {
    /// Send a message via this bridge
    async fn send_message(
        &self,
        message: CrossChainMessage,
        route: Route,
    ) -> RouterResult<MessageReceipt>;

    /// Verify message delivery
    async fn verify_delivery(
        &self,
        message_id: H256,
    ) -> RouterResult<bool>;

    /// Get delivery proof
    async fn get_proof(
        &self,
        message_id: H256,
    ) -> RouterResult<Vec<u8>>;

    /// Estimate delivery time
    async fn estimate_delivery_time(
        &self,
        source: u64,
        dest: u64,
    ) -> RouterResult<u64>;

    /// Estimate fees
    async fn estimate_fees(
        &self,
        message_size: usize,
        source: u64,
        dest: u64,
    ) -> RouterResult<U256>;
}

/// Example LayerZero adapter implementation
pub struct LayerZeroAdapter {
    endpoints: HashMap<u64, Address>,
    relayers: Vec<Address>,
}

#[async_trait]
impl BridgeAdapter for LayerZeroAdapter {
    async fn send_message(
        &self,
        message: CrossChainMessage,
        route: Route,
    ) -> RouterResult<MessageReceipt> {
        // Encode message for LayerZero
        let payload = self.encode_message(&message)?;

        // Get endpoint for source chain
        let endpoint = self
            .endpoints
            .get(&message.source_chain)
            .ok_or(RouterError::ChainNotSupported(message.source_chain))?;

        // Submit to LayerZero endpoint
        // This would involve actual contract interaction
        let tx_hash = self.submit_to_endpoint(endpoint, &message, &payload).await?;

        Ok(MessageReceipt {
            message_id: message.message_id,
            source_tx: Some(tx_hash),
            dest_tx: None,
            status: MessageStatus::Pending,
            timestamp: message.timestamp,
            gas_used: U256::zero(),
            fee_paid: route.base_fee,
        })
    }

    async fn verify_delivery(&self, message_id: H256) -> RouterResult<bool> {
        // Check with LayerZero relayer
        // Implementation would query endpoint contract
        Ok(true)
    }

    async fn get_proof(&self, message_id: H256) -> RouterResult<Vec<u8>> {
        // Get proof from LayerZero
        // Implementation would fetch from relayer
        Ok(vec![])
    }

    async fn estimate_delivery_time(
        &self,
        source: u64,
        dest: u64,
    ) -> RouterResult<u64> {
        // LayerZero typical time: 5-15 minutes
        Ok(600)
    }

    async fn estimate_fees(
        &self,
        message_size: usize,
        source: u64,
        dest: u64,
    ) -> RouterResult<U256> {
        // Base fee + per-byte fee
        let base = U256::from(1000000000000000u64); // 0.001 ETH
        let per_byte = U256::from(1000000000u64); // 0.000000001 ETH
        Ok(base + per_byte * U256::from(message_size))
    }
}

impl LayerZeroAdapter {
    fn encode_message(&self, message: &CrossChainMessage) -> RouterResult<Vec<u8>> {
        bincode::serialize(message)
            .map_err(|e| RouterError::InvalidMessage(e.to_string()))
    }

    async fn submit_to_endpoint(
        &self,
        endpoint: &Address,
        message: &CrossChainMessage,
        payload: &[u8],
    ) -> RouterResult<H256> {
        // Contract interaction would go here
        // For now, return mock tx hash
        Ok(H256::zero())
    }
}
```

#### 4. Message Queue and Retry Logic

```rust
pub struct MessageQueue {
    /// Pending messages
    pending: Arc<RwLock<Vec<CrossChainMessage>>>,

    /// Failed messages awaiting retry
    retry_queue: Arc<RwLock<Vec<(CrossChainMessage, u8)>>>, // (message, retry_count)

    /// Max retry attempts
    max_retries: u8,

    /// Retry delay (seconds)
    retry_delay: u64,

    /// Router reference
    router: Arc<MessageRouter>,
}

impl MessageQueue {
    pub fn new(router: Arc<MessageRouter>) -> Self {
        Self {
            pending: Arc::new(RwLock::new(Vec::new())),
            retry_queue: Arc::new(RwLock::new(Vec::new())),
            max_retries: 3,
            retry_delay: 60,
            router,
        }
    }

    /// Enqueue a message
    pub async fn enqueue(&self, message: CrossChainMessage) {
        let mut pending = self.pending.write().await;
        pending.push(message);
    }

    /// Process message queue
    pub async fn process_queue(&self) -> RouterResult<()> {
        // Process pending messages
        {
            let mut pending = self.pending.write().await;
            let messages: Vec<_> = pending.drain(..).collect();

            for message in messages {
                match self.router.send_message(message.clone()).await {
                    Ok(_) => {
                        // Success
                    }
                    Err(_) => {
                        // Add to retry queue
                        let mut retry = self.retry_queue.write().await;
                        retry.push((message, 0));
                    }
                }
            }
        }

        // Process retry queue
        self.process_retries().await?;

        Ok(())
    }

    async fn process_retries(&self) -> RouterResult<()> {
        let mut retry = self.retry_queue.write().await;
        let mut still_retrying = Vec::new();

        for (message, retry_count) in retry.drain(..) {
            if retry_count >= self.max_retries {
                // Max retries exceeded, give up
                continue;
            }

            match self.router.send_message(message.clone()).await {
                Ok(_) => {
                    // Success, don't re-add
                }
                Err(_) => {
                    // Failed, increment retry count
                    still_retrying.push((message, retry_count + 1));
                }
            }
        }

        *retry = still_retrying;
        Ok(())
    }

    /// Start background processing
    pub async fn start_processing(&self) {
        let queue = Arc::new(self.clone());

        tokio::spawn(async move {
            loop {
                if let Err(e) = queue.process_queue().await {
                    eprintln!("Queue processing error: {}", e);
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(queue.retry_delay)).await;
            }
        });
    }
}

impl Clone for MessageQueue {
    fn clone(&self) -> Self {
        Self {
            pending: Arc::clone(&self.pending),
            retry_queue: Arc::clone(&self.retry_queue),
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
            router: Arc::clone(&self.router),
        }
    }
}
```

## Message Flow

### 1. Intent Execution Flow

```
1. User submits intent on Chain A
2. Intent engine creates IntentExecutionMessage
3. Router selects best route (e.g., LayerZero)
4. Message sent to Chain B
5. Solver executes intent on Chain B
6. Acknowledgment sent back to Chain A
7. Intent marked as complete
```

### 2. State Synchronization Flow

```
1. Pool state changes on Chain A
2. StateSyncMessage created with new state
3. Message broadcast to all chains with pool presence
4. Receivers verify and update virtual pool state
5. Acknowledgments collected
```

## Security Considerations

### 1. Message Authentication
- Signature verification on both source and dest
- Nonce-based replay protection
- Expiry timestamps

### 2. Delivery Guarantees
- At-least-once delivery with idempotency
- Duplicate detection
- Acknowledgment mechanism

### 3. Attack Prevention
- Rate limiting per sender
- Message size limits
- Gas limit validation

## Performance Optimizations

### 1. Batching
- Combine multiple messages into single batch
- Reduce per-message overhead
- Lower total fees

### 2. Compression
- Compress message payloads
- Reduce bandwidth and fees
- Faster transmission

### 3. Prioritization
- Priority queue for urgent messages
- Deadline-aware scheduling
- Dynamic fee adjustment

## Testing Strategy

### Unit Tests
- Message serialization/deserialization
- Route scoring
- Retry logic

### Integration Tests
- End-to-end message delivery
- Multi-bridge scenarios
- Failure recovery

### Chaos Tests
- Network partitions
- Bridge downtime
- High load conditions
