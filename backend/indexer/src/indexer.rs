use ethers::{
    providers::{Provider, Http, Ws, StreamExt, Middleware},
    types::{Filter, Log, Block, Transaction, H256, U64, Address},
    abi::AbiDecode,
};
use tokio::{
    sync::{mpsc, broadcast, RwLock},
    time::{interval, Duration, sleep},
    task::JoinHandle,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::Instant,
};
use futures::future::join_all;

use crate::{
    config::IndexerConfig,
    storage::IndexerStorage,
    events::EventProcessor,
    error::{Result, IndexerError},
    metrics::IndexerMetrics,
    *,
};

// Main blockchain indexer
pub struct BlockchainIndexer {
    config: IndexerConfig,
    storage: Arc<IndexerStorage>,
    event_processor: Arc<EventProcessor>,
    metrics: Arc<IndexerMetrics>,
    chain_handlers: Arc<RwLock<HashMap<u64, ChainIndexer>>>,
    event_broadcaster: broadcast::Sender<IndexedEvent>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: mpsc::Receiver<()>,
}

impl BlockchainIndexer {
    pub async fn new(config: IndexerConfig) -> Result<Self> {
        let storage = Arc::new(IndexerStorage::new(&config.database_url).await?);
        let event_processor = Arc::new(EventProcessor::new(storage.clone()));
        let metrics = Arc::new(IndexerMetrics::new());
        
        let (event_broadcaster, _) = broadcast::channel(10000);
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        
        Ok(Self {
            config,
            storage,
            event_processor,
            metrics,
            chain_handlers: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster,
            shutdown_tx,
            shutdown_rx,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting blockchain indexer");
        
        // Initialize database
        self.storage.initialize().await?;
        
        // Start chain indexers
        let mut tasks = Vec::new();
        
        for chain_config in &self.config.chains {
            if chain_config.enabled {
                let chain_indexer = ChainIndexer::new(
                    chain_config.clone(),
                    self.storage.clone(),
                    self.event_processor.clone(),
                    self.metrics.clone(),
                    self.event_broadcaster.clone(),
                ).await?;
                
                // Start chain indexer
                let task = tokio::spawn(async move {
                    chain_indexer.start().await
                });
                
                tasks.push(task);
                
                self.chain_handlers.write().await.insert(
                    chain_config.chain_id,
                    chain_indexer,
                );
                
                tracing::info!("Started indexer for chain: {}", chain_config.name);
            }
        }
        
        // Start metrics collection
        let metrics_task = self.start_metrics_collection();
        tasks.push(metrics_task);
        
        // Start health monitoring
        let health_task = self.start_health_monitoring();
        tasks.push(health_task);
        
        tracing::info!("All indexer services started");
        
        // Wait for shutdown signal or task completion
        tokio::select! {
            _ = self.shutdown_rx.recv() => {
                tracing::info!("Shutdown signal received");
            }
            result = join_all(tasks) => {
                tracing::error!("Indexer task completed unexpectedly: {:?}", result);
            }
        }
        
        self.shutdown().await?;
        Ok(())
    }
    
    pub async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down blockchain indexer");
        
        // Stop all chain indexers
        let handlers = self.chain_handlers.read().await;
        for (chain_id, handler) in handlers.iter() {
            tracing::info!("Stopping indexer for chain: {}", chain_id);
            handler.stop().await?;
        }
        
        Ok(())
    }
    
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<IndexedEvent> {
        self.event_broadcaster.subscribe()
    }
    
    pub async fn get_chain_state(&self, chain_id: u64) -> Result<Option<ChainState>> {
        self.storage.get_chain_state(chain_id).await
    }
    
    pub async fn get_stats(&self) -> Result<IndexerStats> {
        self.storage.get_stats().await
    }
    
    fn start_metrics_collection(&self) -> JoinHandle<()> {
        let metrics = self.metrics.clone();
        let storage = self.storage.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Collect metrics every minute
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::collect_metrics(&metrics, &storage).await {
                    tracing::warn!("Failed to collect metrics: {}", e);
                }
            }
        })
    }
    
    fn start_health_monitoring(&self) -> JoinHandle<()> {
        let chain_handlers = self.chain_handlers.clone();
        let storage = self.storage.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check health every 30 seconds
            
            loop {
                interval.tick().await;
                
                if let Err(e) = Self::monitor_health(&chain_handlers, &storage).await {
                    tracing::warn!("Health monitoring failed: {}", e);
                }
            }
        })
    }
    
    async fn collect_metrics(
        metrics: &IndexerMetrics,
        storage: &IndexerStorage,
    ) -> Result<()> {
        let stats = storage.get_stats().await?;
        
        // Update Prometheus metrics
        metrics.set_total_events_indexed(stats.total_events_indexed);
        metrics.set_events_per_second(stats.events_per_second);
        metrics.set_total_intents_processed(stats.total_intents_processed);
        
        for (chain_id, chain_stats) in stats.chains {
            metrics.set_chain_blocks_indexed(chain_id, chain_stats.blocks_indexed);
            metrics.set_chain_events_indexed(chain_id, chain_stats.events_indexed);
            metrics.set_chain_sync_progress(chain_id, chain_stats.sync_progress);
            metrics.set_chain_blocks_behind(chain_id, chain_stats.blocks_behind);
        }
        
        Ok(())
    }
    
    async fn monitor_health(
        _chain_handlers: &Arc<RwLock<HashMap<u64, ChainIndexer>>>,
        _storage: &IndexerStorage,
    ) -> Result<()> {
        // TODO: Implement health monitoring logic
        // - Check if chains are syncing
        // - Monitor error rates
        // - Check database connectivity
        // - Verify event processing is working
        
        Ok(())
    }
}

// Chain-specific indexer
#[derive(Clone)]
pub struct ChainIndexer {
    config: ChainIndexerConfig,
    provider: Arc<Provider<Http>>,
    ws_provider: Option<Arc<Provider<Ws>>>,
    storage: Arc<IndexerStorage>,
    event_processor: Arc<EventProcessor>,
    metrics: Arc<IndexerMetrics>,
    event_broadcaster: broadcast::Sender<IndexedEvent>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl ChainIndexer {
    pub async fn new(
        config: ChainIndexerConfig,
        storage: Arc<IndexerStorage>,
        event_processor: Arc<EventProcessor>,
        metrics: Arc<IndexerMetrics>,
        event_broadcaster: broadcast::Sender<IndexedEvent>,
    ) -> Result<Self> {
        // Create HTTP provider
        let provider = Arc::new(
            Provider::<Http>::try_from(&config.rpc_url)
                .map_err(|e| IndexerError::ProviderError(e.to_string()))?
        );
        
        // Create WebSocket provider if available
        let ws_provider = if let Some(ws_url) = &config.ws_url {
            match Provider::<Ws>::connect(ws_url).await {
                Ok(ws) => Some(Arc::new(ws)),
                Err(e) => {
                    tracing::warn!("Failed to connect to WebSocket for chain {}: {}", config.chain_id, e);
                    None
                }
            }
        } else {
            None
        };
        
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        
        Ok(Self {
            config,
            provider,
            ws_provider,
            storage,
            event_processor,
            metrics,
            event_broadcaster,
            shutdown_tx,
            shutdown_rx: Some(shutdown_rx),
        })
    }
    
    pub async fn start(mut self) -> Result<()> {
        tracing::info!("Starting chain indexer for chain: {}", self.config.chain_id);
        
        // Get current chain state
        let mut chain_state = self.get_or_create_chain_state().await?;
        
        // Start real-time event monitoring if WebSocket is available
        let realtime_task = if let Some(ws_provider) = &self.ws_provider {
            Some(self.start_realtime_monitoring(ws_provider.clone()))
        } else {
            None
        };
        
        // Start historical sync
        let historical_task = self.start_historical_sync(chain_state.clone());
        
        // Start block monitoring
        let block_monitoring_task = self.start_block_monitoring();
        
        let mut tasks = vec![historical_task, block_monitoring_task];
        if let Some(task) = realtime_task {
            tasks.push(task);
        }
        
        // Wait for shutdown or task completion
        tokio::select! {
            _ = self.shutdown_rx.as_mut().unwrap().recv() => {
                tracing::info!("Chain indexer shutdown for chain: {}", self.config.chain_id);
            }
            result = join_all(tasks) => {
                tracing::error!("Chain indexer task failed for chain {}: {:?}", self.config.chain_id, result);
            }
        }
        
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        self.shutdown_tx.send(()).await
            .map_err(|_| IndexerError::Internal("Failed to send shutdown signal".to_string()))?;
        Ok(())
    }
    
    async fn get_or_create_chain_state(&self) -> Result<ChainState> {
        if let Some(state) = self.storage.get_chain_state(self.config.chain_id).await? {
            Ok(state)
        } else {
            // Create initial chain state
            let latest_block = self.provider.get_block_number().await
                .map_err(|e| IndexerError::ProviderError(e.to_string()))?;
            
            let state = ChainState {
                chain_id: self.config.chain_id,
                latest_block: latest_block.as_u64(),
                latest_block_hash: H256::zero(), // Will be updated
                indexed_block: self.config.start_block.max(latest_block.as_u64().saturating_sub(1000)), // Start from recent history
                confirmation_blocks: self.config.confirmation_blocks,
                is_synced: false,
                last_update: chrono::Utc::now(),
            };
            
            self.storage.save_chain_state(&state).await?;
            Ok(state)
        }
    }
    
    fn start_historical_sync(&self, mut chain_state: ChainState) -> JoinHandle<Result<()>> {
        let provider = self.provider.clone();
        let storage = self.storage.clone();
        let event_processor = self.event_processor.clone();
        let config = self.config.clone();
        let event_broadcaster = self.event_broadcaster.clone();
        
        tokio::spawn(async move {
            tracing::info!("Starting historical sync for chain: {}", config.chain_id);
            
            loop {
                // Get latest block
                let latest_block = provider.get_block_number().await
                    .map_err(|e| IndexerError::ProviderError(e.to_string()))?;
                
                let target_block = latest_block.as_u64().saturating_sub(config.confirmation_blocks);
                
                if chain_state.indexed_block >= target_block {
                    // We're caught up, wait a bit
                    chain_state.is_synced = true;
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
                
                // Process a batch of blocks
                let end_block = (chain_state.indexed_block + config.batch_size).min(target_block);
                
                tracing::debug!(
                    "Processing blocks {} to {} for chain {}",
                    chain_state.indexed_block + 1,
                    end_block,
                    config.chain_id
                );
                
                let start_time = Instant::now();
                
                for block_number in (chain_state.indexed_block + 1)..=end_block {
                    if let Err(e) = Self::process_block(
                        &provider,
                        &storage,
                        &event_processor,
                        &config,
                        &event_broadcaster,
                        block_number,
                    ).await {
                        tracing::error!(
                            "Failed to process block {} for chain {}: {}",
                            block_number,
                            config.chain_id,
                            e
                        );
                        
                        // Wait before retrying
                        sleep(Duration::from_secs(5)).await;
                        continue;
                    }
                    
                    chain_state.indexed_block = block_number;
                }
                
                chain_state.latest_block = latest_block.as_u64();
                chain_state.last_update = chrono::Utc::now();
                
                // Save updated state
                if let Err(e) = storage.save_chain_state(&chain_state).await {
                    tracing::error!("Failed to save chain state: {}", e);
                }
                
                let processing_time = start_time.elapsed();
                tracing::debug!(
                    "Processed {} blocks for chain {} in {:?}",
                    end_block - chain_state.indexed_block + config.batch_size,
                    config.chain_id,
                    processing_time
                );
                
                // Small delay to prevent overwhelming the RPC
                sleep(Duration::from_millis(100)).await;
            }
        })
    }
    
    fn start_realtime_monitoring(&self, ws_provider: Arc<Provider<Ws>>) -> JoinHandle<Result<()>> {
        let storage = self.storage.clone();
        let event_processor = self.event_processor.clone();
        let config = self.config.clone();
        let event_broadcaster = self.event_broadcaster.clone();
        
        tokio::spawn(async move {
            tracing::info!("Starting real-time monitoring for chain: {}", config.chain_id);
            
            // Subscribe to new blocks
            let mut stream = ws_provider.subscribe_blocks().await
                .map_err(|e| IndexerError::ProviderError(e.to_string()))?;
            
            while let Some(block) = stream.next().await {
                if let Some(block_number) = block.number {
                    // Process the new block
                    if let Err(e) = Self::process_block(
                        &ws_provider,
                        &storage,
                        &event_processor,
                        &config,
                        &event_broadcaster,
                        block_number.as_u64(),
                    ).await {
                        tracing::error!(
                            "Failed to process real-time block {} for chain {}: {}",
                            block_number,
                            config.chain_id,
                            e
                        );
                    }
                }
            }
            
            tracing::warn!("Real-time monitoring stream ended for chain: {}", config.chain_id);
            Ok(())
        })
    }
    
    fn start_block_monitoring(&self) -> JoinHandle<Result<()>> {
        let provider = self.provider.clone();
        let storage = self.storage.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(15)); // Check for new blocks every 15 seconds
            
            loop {
                interval.tick().await;
                
                if let Ok(latest_block) = provider.get_block_number().await {
                    // Update chain state with latest block info
                    if let Ok(Some(mut chain_state)) = storage.get_chain_state(config.chain_id).await {
                        chain_state.latest_block = latest_block.as_u64();
                        chain_state.last_update = chrono::Utc::now();
                        
                        if let Err(e) = storage.save_chain_state(&chain_state).await {
                            tracing::warn!("Failed to update chain state: {}", e);
                        }
                    }
                }
            }
        })
    }
    
    async fn process_block<P: Middleware + 'static>(
        provider: &Arc<P>,
        storage: &Arc<IndexerStorage>,
        event_processor: &Arc<EventProcessor>,
        config: &ChainIndexerConfig,
        event_broadcaster: &broadcast::Sender<IndexedEvent>,
        block_number: u64,
    ) -> Result<()> {
        // Get block with transactions
        let block = provider.get_block_with_txs(block_number).await
            .map_err(|e| IndexerError::ProviderError(e.to_string()))?;
        
        let block = block.ok_or_else(|| 
            IndexerError::DataNotFound(format!("Block {} not found", block_number))
        )?;
        
        // Create event filters for our contracts
        let filters = Self::create_event_filters(config, block_number);
        
        // Get logs for this block
        for filter in filters {
            let logs = provider.get_logs(&filter).await
                .map_err(|e| IndexerError::ProviderError(e.to_string()))?;
            
            // Process each log
            for log in logs {
                let indexed_event = event_processor.process_log(
                    config.chain_id,
                    &log,
                    block.timestamp,
                ).await?;
                
                // Store the event
                storage.store_event(&indexed_event).await?;
                
                // Broadcast to subscribers
                if let Err(e) = event_broadcaster.send(indexed_event) {
                    tracing::warn!("Failed to broadcast event: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    fn create_event_filters(config: &ChainIndexerConfig, block_number: u64) -> Vec<Filter> {
        let mut filters = Vec::new();
        
        // Intent contract events
        filters.push(
            Filter::new()
                .address(config.contracts.intents_contract)
                .from_block(block_number)
                .to_block(block_number)
        );
        
        // Orbital AMM contract events
        filters.push(
            Filter::new()
                .address(config.contracts.orbital_amm_contract)
                .from_block(block_number)
                .to_block(block_number)
        );
        
        // Bridge contract events
        filters.push(
            Filter::new()
                .address(config.contracts.bridge_contract)
                .from_block(block_number)
                .to_block(block_number)
        );
        
        // Solver registry events (if configured)
        if let Some(solver_registry) = config.contracts.solver_registry {
            filters.push(
                Filter::new()
                    .address(solver_registry)
                    .from_block(block_number)
                    .to_block(block_number)
            );
        }
        
        filters
    }
}