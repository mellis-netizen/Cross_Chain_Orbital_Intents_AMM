use crate::{intent::*, state::EngineState, ChainConfig, Result, EngineError};
use ethers::{
    prelude::*,
    providers::{Provider, Http},
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
    time::timeout,
};

pub struct IntentExecutor {
    chains: Arc<RwLock<HashMap<u64, ChainState>>>,
    state: Arc<EngineState>,
    intent_queue: mpsc::UnboundedSender<(H256, Intent)>,
    executor_handle: RwLock<Option<JoinHandle<()>>>,
}

struct ChainState {
    config: ChainConfig,
    provider: Arc<Provider<Http>>,
}

impl IntentExecutor {
    pub async fn new(chains: Vec<ChainConfig>, state: Arc<EngineState>) -> Result<Self> {
        let mut chain_map = HashMap::new();
        
        for config in chains {
            let provider = Provider::<Http>::try_from(&config.rpc_url)
                .map_err(|e| EngineError::BridgeError(e.to_string()))?;
            
            chain_map.insert(config.chain_id, ChainState {
                config: config.clone(),
                provider: Arc::new(provider),
            });
        }
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            chains: Arc::new(RwLock::new(chain_map)),
            state,
            intent_queue: tx,
            executor_handle: RwLock::new(None),
        })
    }
    
    pub async fn queue_intent(&self, intent_id: H256, intent: Intent) -> Result<()> {
        self.intent_queue.send((intent_id, intent))
            .map_err(|_| EngineError::ExecutionFailed("Failed to queue intent".to_string()))?;
        Ok(())
    }
    
    pub async fn add_chain(&self, config: ChainConfig) -> Result<()> {
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .map_err(|e| EngineError::BridgeError(e.to_string()))?;
        
        let mut chains = self.chains.write().await;
        chains.insert(config.chain_id, ChainState {
            config: config.clone(),
            provider: Arc::new(provider),
        });
        
        Ok(())
    }
    
    pub async fn start(&self) -> Result<()> {
        let mut handle = self.executor_handle.write().await;
        
        if handle.is_some() {
            return Ok(());
        }
        
        let chains = self.chains.clone();
        let state = self.state.clone();
        let mut rx = {
            let (tx, rx) = mpsc::unbounded_channel();
            let _ = std::mem::replace(&mut self.intent_queue, tx);
            rx
        };
        
        let executor_task = tokio::spawn(async move {
            while let Some((intent_id, intent)) = rx.recv().await {
                let chains = chains.clone();
                let state = state.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = execute_intent(intent_id, intent, chains, state).await {
                        tracing::error!("Failed to execute intent {}: {:?}", intent_id, e);
                    }
                });
            }
        });
        
        *handle = Some(executor_task);
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        let mut handle = self.executor_handle.write().await;
        
        if let Some(task) = handle.take() {
            task.abort();
        }
        
        Ok(())
    }
}

async fn execute_intent(
    intent_id: H256,
    intent: Intent,
    chains: Arc<RwLock<HashMap<u64, ChainState>>>,
    state: Arc<EngineState>,
) -> Result<()> {
    state.update_intent_status(intent_id, IntentStatus::Executing).await?;
    
    let chains_read = chains.read().await;
    
    let source_chain = chains_read.get(&intent.source_chain_id)
        .ok_or_else(|| EngineError::ChainNotSupported(intent.source_chain_id))?;
    
    let dest_chain = chains_read.get(&intent.dest_chain_id)
        .ok_or_else(|| EngineError::ChainNotSupported(intent.dest_chain_id))?;
    
    match execute_cross_chain_swap(
        &intent,
        source_chain,
        dest_chain,
    ).await {
        Ok(dest_amount) => {
            state.complete_intent(intent_id, dest_amount).await?;
            Ok(())
        }
        Err(e) => {
            state.fail_intent(intent_id).await?;
            Err(e)
        }
    }
}

async fn execute_cross_chain_swap(
    intent: &Intent,
    source_chain: &ChainState,
    dest_chain: &ChainState,
) -> Result<U256> {
    tracing::info!(
        "Starting cross-chain swap execution for intent: {:?}",
        intent.compute_id()
    );

    // Pre-task hook for coordination
    let _ = execute_hook("pre-task", &format!("execute_cross_chain_swap_{}", intent.compute_id())).await;

    // Step 1: Lock source assets on source chain
    let lock_result = lock_source_assets(intent, source_chain).await?;
    tracing::info!("Assets locked on source chain: tx_hash={:?}", lock_result.tx_hash);

    // Hook: notify asset lock
    let _ = execute_hook(
        "notify",
        &format!("Assets locked: {} tokens on chain {}", intent.source_amount, intent.source_chain_id)
    ).await;

    // Step 2: Get optimal route from solver/optimizer
    let route = get_execution_route(intent, source_chain, dest_chain).await?;
    tracing::info!("Optimal route found: {} hops", route.hops.len());

    // Hook: store route in memory
    let _ = execute_hook(
        "post-edit",
        &format!("route_{}_{}", intent.source_chain_id, intent.dest_chain_id)
    ).await;

    // Step 3: Execute swap on destination chain via bridge
    let bridge_tx = execute_via_bridge(intent, &route, source_chain, dest_chain).await?;
    tracing::info!("Bridge transaction initiated: tx_hash={:?}", bridge_tx.tx_hash);

    // Step 4: Wait for execution confirmation with timeout
    let execution_result = timeout(
        Duration::from_secs(300), // 5 minute timeout
        wait_for_execution(&bridge_tx, dest_chain)
    )
    .await
    .map_err(|_| EngineError::ExecutionFailed("Execution timeout exceeded".to_string()))??;

    tracing::info!("Execution confirmed: amount={}", execution_result.dest_amount);

    // Step 5: Verify execution proof
    verify_execution_proof(&execution_result, dest_chain).await?;
    tracing::info!("Execution proof verified successfully");

    // Hook: notify successful execution
    let _ = execute_hook(
        "notify",
        &format!("Swap executed: {} tokens received on chain {}", execution_result.dest_amount, intent.dest_chain_id)
    ).await;

    // Post-task hook
    let _ = execute_hook("post-task", &format!("intent_{}", intent.compute_id())).await;

    Ok(execution_result.dest_amount)
}

/// Lock source assets on the source chain
async fn lock_source_assets(
    intent: &Intent,
    source_chain: &ChainState,
) -> Result<LockResult> {
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    loop {
        match try_lock_assets(intent, source_chain).await {
            Ok(result) => return Ok(result),
            Err(e) if retry_count < MAX_RETRIES => {
                retry_count += 1;
                tracing::warn!(
                    "Failed to lock assets (attempt {}/{}): {:?}. Retrying...",
                    retry_count,
                    MAX_RETRIES,
                    e
                );
                tokio::time::sleep(Duration::from_secs(2u64.pow(retry_count))).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn try_lock_assets(
    intent: &Intent,
    source_chain: &ChainState,
) -> Result<LockResult> {
    // Get the intents contract
    let intents_contract = source_chain.config.intents_contract;

    // Build the lock transaction
    let lock_tx = build_lock_transaction(intent, intents_contract)?;

    // Send transaction
    let pending_tx = source_chain.provider
        .send_transaction(lock_tx, None)
        .await
        .map_err(|e| EngineError::ExecutionFailed(format!("Failed to send lock tx: {}", e)))?;

    // Wait for confirmation
    let receipt = pending_tx
        .confirmations(source_chain.config.confirmation_blocks as usize)
        .await
        .map_err(|e| EngineError::ExecutionFailed(format!("Failed to confirm lock tx: {}", e)))?
        .ok_or_else(|| EngineError::ExecutionFailed("Lock transaction failed".to_string()))?;

    Ok(LockResult {
        tx_hash: receipt.transaction_hash,
        block_number: receipt.block_number.unwrap_or_default().as_u64(),
        lock_id: H256::from_slice(&receipt.logs[0].topics[1].as_bytes()),
    })
}

fn build_lock_transaction(
    intent: &Intent,
    intents_contract: Address,
) -> Result<TransactionRequest> {
    // Encode the lockIntent function call
    // function lockIntent(Intent calldata intent) external returns (bytes32)
    let function_data = ethers::abi::encode(&[
        ethers::abi::Token::Address(intent.user),
        ethers::abi::Token::Uint(intent.source_chain_id.into()),
        ethers::abi::Token::Uint(intent.dest_chain_id.into()),
        ethers::abi::Token::Address(intent.source_token),
        ethers::abi::Token::Address(intent.dest_token),
        ethers::abi::Token::Uint(intent.source_amount),
        ethers::abi::Token::Uint(intent.min_dest_amount),
        ethers::abi::Token::Uint(intent.deadline.into()),
        ethers::abi::Token::Uint(intent.nonce),
    ]);

    let function_selector = ethers::utils::keccak256(b"lockIntent((address,uint256,uint256,address,address,uint256,uint256,uint64,uint256))");
    let mut calldata = function_selector[..4].to_vec();
    calldata.extend(function_data);

    Ok(TransactionRequest::new()
        .to(intents_contract)
        .data(calldata))
}

/// Get optimal execution route from solver/optimizer
async fn get_execution_route(
    intent: &Intent,
    source_chain: &ChainState,
    dest_chain: &ChainState,
) -> Result<ExecutionRoute> {
    tracing::debug!("Finding optimal route for intent");

    // Query the Orbital AMM for the best route
    let route = query_orbital_amm_route(intent, source_chain, dest_chain).await?;

    // Validate route profitability
    if route.estimated_output < intent.min_dest_amount {
        return Err(EngineError::ExecutionFailed(
            format!(
                "Route output {} is less than minimum required {}",
                route.estimated_output,
                intent.min_dest_amount
            )
        ));
    }

    Ok(route)
}

async fn query_orbital_amm_route(
    intent: &Intent,
    source_chain: &ChainState,
    dest_chain: &ChainState,
) -> Result<ExecutionRoute> {
    // In production, this would query the actual Orbital AMM contract
    // For now, we create a simple direct route

    Ok(ExecutionRoute {
        hops: vec![
            RouteHop {
                chain_id: intent.source_chain_id,
                pool: source_chain.config.orbital_amm_contract,
                token_in: intent.source_token,
                token_out: intent.dest_token,
                amount_in: intent.source_amount,
            }
        ],
        estimated_output: intent.min_dest_amount,
        estimated_gas: U256::from(500000),
        bridge_protocol: "LayerZero".to_string(),
    })
}

/// Execute swap via bridge protocol
async fn execute_via_bridge(
    intent: &Intent,
    route: &ExecutionRoute,
    source_chain: &ChainState,
    dest_chain: &ChainState,
) -> Result<BridgeTransaction> {
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    loop {
        match try_execute_bridge(intent, route, source_chain, dest_chain).await {
            Ok(result) => return Ok(result),
            Err(e) if retry_count < MAX_RETRIES => {
                retry_count += 1;
                tracing::warn!(
                    "Failed to execute bridge (attempt {}/{}): {:?}. Retrying...",
                    retry_count,
                    MAX_RETRIES,
                    e
                );
                tokio::time::sleep(Duration::from_secs(3u64.pow(retry_count))).await;
            }
            Err(e) => return Err(e),
        }
    }
}

async fn try_execute_bridge(
    intent: &Intent,
    route: &ExecutionRoute,
    source_chain: &ChainState,
    dest_chain: &ChainState,
) -> Result<BridgeTransaction> {
    let bridge_contract = source_chain.config.bridge_contract;

    // Build cross-chain message payload
    let payload = build_bridge_payload(intent, route)?;

    // Encode bridge function call
    let calldata = encode_bridge_call(
        intent.dest_chain_id,
        dest_chain.config.intents_contract,
        payload,
    )?;

    // Send bridge transaction
    let tx = TransactionRequest::new()
        .to(bridge_contract)
        .data(calldata)
        .value(route.estimated_gas); // Bridge fee

    let pending_tx = source_chain.provider
        .send_transaction(tx, None)
        .await
        .map_err(|e| EngineError::BridgeError(format!("Failed to send bridge tx: {}", e)))?;

    let receipt = pending_tx
        .confirmations(source_chain.config.confirmation_blocks as usize)
        .await
        .map_err(|e| EngineError::BridgeError(format!("Failed to confirm bridge tx: {}", e)))?
        .ok_or_else(|| EngineError::BridgeError("Bridge transaction failed".to_string()))?;

    Ok(BridgeTransaction {
        tx_hash: receipt.transaction_hash,
        block_number: receipt.block_number.unwrap_or_default().as_u64(),
        message_id: extract_message_id(&receipt)?,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

fn build_bridge_payload(intent: &Intent, route: &ExecutionRoute) -> Result<Vec<u8>> {
    // Encode the swap execution parameters
    let encoded = ethers::abi::encode(&[
        ethers::abi::Token::Bytes32(intent.compute_id().into()),
        ethers::abi::Token::Address(intent.user),
        ethers::abi::Token::Address(intent.dest_token),
        ethers::abi::Token::Uint(route.estimated_output),
        ethers::abi::Token::Uint(intent.deadline.into()),
    ]);

    Ok(encoded)
}

fn encode_bridge_call(
    dest_chain_id: u64,
    dest_contract: Address,
    payload: Vec<u8>,
) -> Result<Vec<u8>> {
    // function sendCrossChainMessage(uint256 destChain, address destContract, bytes calldata payload)
    let function_selector = ethers::utils::keccak256(b"sendCrossChainMessage(uint256,address,bytes)");

    let encoded = ethers::abi::encode(&[
        ethers::abi::Token::Uint(dest_chain_id.into()),
        ethers::abi::Token::Address(dest_contract),
        ethers::abi::Token::Bytes(payload),
    ]);

    let mut calldata = function_selector[..4].to_vec();
    calldata.extend(encoded);

    Ok(calldata)
}

fn extract_message_id(receipt: &TransactionReceipt) -> Result<H256> {
    // Extract MessageSent event topic
    receipt.logs
        .iter()
        .find(|log| {
            !log.topics.is_empty() &&
            log.topics[0] == ethers::utils::keccak256(b"MessageSent(bytes32,uint256,address,bytes)").into()
        })
        .and_then(|log| log.topics.get(1))
        .copied()
        .ok_or_else(|| EngineError::BridgeError("MessageSent event not found".to_string()))
}

/// Wait for execution confirmation on destination chain
async fn wait_for_execution(
    bridge_tx: &BridgeTransaction,
    dest_chain: &ChainState,
) -> Result<ExecutionResult> {
    const POLL_INTERVAL_SECS: u64 = 10;
    const MAX_POLLS: u32 = 30; // 5 minutes total

    for poll_count in 0..MAX_POLLS {
        tracing::debug!("Polling for execution confirmation (attempt {}/{})", poll_count + 1, MAX_POLLS);

        match check_execution_status(bridge_tx.message_id, dest_chain).await {
            Ok(Some(result)) => return Ok(result),
            Ok(None) => {
                tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            }
            Err(e) => {
                tracing::warn!("Error checking execution status: {:?}", e);
                tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
            }
        }
    }

    Err(EngineError::ExecutionFailed("Execution not confirmed within timeout".to_string()))
}

async fn check_execution_status(
    message_id: H256,
    dest_chain: &ChainState,
) -> Result<Option<ExecutionResult>> {
    // Query the destination chain for execution status
    let intents_contract = dest_chain.config.intents_contract;

    // Build call to getExecutionStatus(bytes32 messageId)
    let function_selector = ethers::utils::keccak256(b"getExecutionStatus(bytes32)");
    let mut calldata = function_selector[..4].to_vec();
    calldata.extend(ethers::abi::encode(&[ethers::abi::Token::Bytes32(message_id.into())]));

    let call = CallRequest {
        to: Some(intents_contract),
        data: Some(calldata.into()),
        ..Default::default()
    };

    let result = dest_chain.provider
        .call(&call.into(), None)
        .await
        .map_err(|e| EngineError::BridgeError(format!("Failed to call getExecutionStatus: {}", e)))?;

    // Decode response: (bool executed, uint256 amount, bytes32 txHash)
    if result.len() < 96 {
        return Ok(None);
    }

    let executed = !result[31] == 0;
    if !executed {
        return Ok(None);
    }

    let dest_amount = U256::from_big_endian(&result[32..64]);
    let tx_hash = H256::from_slice(&result[64..96]);

    Ok(Some(ExecutionResult {
        dest_amount,
        tx_hash,
        block_number: dest_chain.provider.get_block_number().await.unwrap_or_default().as_u64(),
    }))
}

/// Verify execution proof using bridge protocol
async fn verify_execution_proof(
    execution_result: &ExecutionResult,
    dest_chain: &ChainState,
) -> Result<()> {
    tracing::debug!("Verifying execution proof for tx: {:?}", execution_result.tx_hash);

    // Get transaction receipt
    let receipt = dest_chain.provider
        .get_transaction_receipt(execution_result.tx_hash)
        .await
        .map_err(|e| EngineError::BridgeError(format!("Failed to get receipt: {}", e)))?
        .ok_or_else(|| EngineError::BridgeError("Receipt not found".to_string()))?;

    // Verify transaction succeeded
    if receipt.status != Some(1.into()) {
        return Err(EngineError::ExecutionFailed("Destination transaction failed".to_string()));
    }

    // Verify sufficient confirmations
    let current_block = dest_chain.provider
        .get_block_number()
        .await
        .map_err(|e| EngineError::BridgeError(format!("Failed to get block number: {}", e)))?
        .as_u64();

    let confirmations = current_block.saturating_sub(execution_result.block_number);

    if confirmations < dest_chain.config.confirmation_blocks {
        return Err(EngineError::ExecutionFailed(
            format!("Insufficient confirmations: {} < {}", confirmations, dest_chain.config.confirmation_blocks)
        ));
    }

    // In production, would verify Merkle proofs, ZK proofs, or optimistic fraud proofs
    // depending on the bridge protocol

    Ok(())
}

/// Execute coordination hooks
async fn execute_hook(hook_type: &str, data: &str) -> Result<()> {
    use std::process::Command;

    let output = Command::new("npx")
        .args(&[
            "claude-flow@alpha",
            "hooks",
            hook_type,
            "--description",
            data,
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            tracing::debug!("Hook {} executed successfully", hook_type);
            Ok(())
        }
        Ok(out) => {
            tracing::warn!(
                "Hook {} failed: {}",
                hook_type,
                String::from_utf8_lossy(&out.stderr)
            );
            Ok(()) // Don't fail execution on hook errors
        }
        Err(e) => {
            tracing::warn!("Failed to execute hook {}: {}", hook_type, e);
            Ok(()) // Don't fail execution on hook errors
        }
    }
}

// Supporting structures
#[derive(Debug, Clone)]
struct LockResult {
    tx_hash: H256,
    block_number: u64,
    lock_id: H256,
}

#[derive(Debug, Clone)]
struct ExecutionRoute {
    hops: Vec<RouteHop>,
    estimated_output: U256,
    estimated_gas: U256,
    bridge_protocol: String,
}

#[derive(Debug, Clone)]
struct RouteHop {
    chain_id: u64,
    pool: Address,
    token_in: Address,
    token_out: Address,
    amount_in: U256,
}

#[derive(Debug, Clone)]
struct BridgeTransaction {
    tx_hash: H256,
    block_number: u64,
    message_id: H256,
    timestamp: u64,
}

#[derive(Debug, Clone)]
struct ExecutionResult {
    dest_amount: U256,
    tx_hash: H256,
    block_number: u64,
}