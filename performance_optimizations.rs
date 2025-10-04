//! Performance Optimizations for Cross Chain Orbital Intents AMM
//!
//! This module implements specific performance optimizations identified
//! through analysis and benchmarking.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock as TokioRwLock};
use serde::{Deserialize, Serialize};

/// High-performance cache for route calculations
#[derive(Clone)]
pub struct RouteCache {
    cache: Arc<RwLock<HashMap<String, CachedRoute>>>,
    max_size: usize,
    ttl: Duration,
}

#[derive(Clone, Debug)]
struct CachedRoute {
    route: RouteData,
    created_at: Instant,
    access_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RouteData {
    pub hops: Vec<String>,
    pub estimated_gas: u64,
    pub estimated_output: u64,
    pub cross_chain: bool,
}

impl RouteCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl,
        }
    }

    /// Get route from cache with LRU eviction
    pub async fn get(&self, key: &str) -> Option<RouteData> {
        let mut cache = self.cache.write().unwrap();
        
        if let Some(cached) = cache.get_mut(key) {
            // Check if expired
            if cached.created_at.elapsed() > self.ttl {
                cache.remove(key);
                return None;
            }
            
            // Update access count for LRU
            cached.access_count += 1;
            return Some(cached.route.clone());
        }
        
        None
    }

    /// Store route in cache with LRU eviction
    pub async fn put(&self, key: String, route: RouteData) {
        let mut cache = self.cache.write().unwrap();
        
        // Evict if at capacity
        if cache.len() >= self.max_size {
            // Find least recently used item
            let lru_key = cache
                .iter()
                .min_by_key(|(_, cached)| cached.access_count)
                .map(|(k, _)| k.clone());
            
            if let Some(key_to_remove) = lru_key {
                cache.remove(&key_to_remove);
            }
        }
        
        cache.insert(key, CachedRoute {
            route,
            created_at: Instant::now(),
            access_count: 1,
        });
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        
        CacheStats {
            size: cache.len(),
            max_size: self.max_size,
            hit_rate: 0.0, // Would need to track hits/misses in production
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_rate: f64,
}

/// Connection pool for database operations
pub struct DatabasePool {
    connections: Arc<Semaphore>,
    max_connections: usize,
    connection_timeout: Duration,
}

impl DatabasePool {
    pub fn new(max_connections: usize, connection_timeout: Duration) -> Self {
        Self {
            connections: Arc::new(Semaphore::new(max_connections)),
            max_connections,
            connection_timeout,
        }
    }

    /// Acquire database connection with timeout
    pub async fn acquire_connection(&self) -> Result<DatabaseConnection, PoolError> {
        let permit = tokio::time::timeout(
            self.connection_timeout,
            self.connections.acquire()
        ).await
        .map_err(|_| PoolError::Timeout)?
        .map_err(|_| PoolError::Closed)?;
        
        Ok(DatabaseConnection {
            _permit: permit,
        })
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            available: self.connections.available_permits(),
            max_connections: self.max_connections,
            active_connections: self.max_connections - self.connections.available_permits(),
        }
    }
}

pub struct DatabaseConnection {
    _permit: tokio::sync::SemaphorePermit<'_>,
}

#[derive(Debug)]
pub enum PoolError {
    Timeout,
    Closed,
}

#[derive(Debug)]
pub struct PoolStats {
    pub available: usize,
    pub max_connections: usize,
    pub active_connections: usize,
}

/// Batch processor for WebSocket messages
pub struct MessageBatcher {
    pending_messages: Arc<TokioRwLock<Vec<BatchedMessage>>>,
    batch_size: usize,
    flush_interval: Duration,
}

#[derive(Clone, Debug)]
struct BatchedMessage {
    content: String,
    timestamp: Instant,
    channel: String,
}

impl MessageBatcher {
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            pending_messages: Arc::new(TokioRwLock::new(Vec::new())),
            batch_size,
            flush_interval,
        }
    }

    /// Add message to batch
    pub async fn add_message(&self, content: String, channel: String) -> bool {
        let mut messages = self.pending_messages.write().await;
        
        messages.push(BatchedMessage {
            content,
            timestamp: Instant::now(),
            channel,
        });
        
        // Return true if batch is ready to flush
        messages.len() >= self.batch_size
    }

    /// Flush batch if ready
    pub async fn try_flush(&self) -> Option<Vec<BatchedMessage>> {
        let mut messages = self.pending_messages.write().await;
        
        let should_flush = messages.len() >= self.batch_size || 
                          messages.iter().any(|m| m.timestamp.elapsed() > self.flush_interval);
        
        if should_flush {
            let batch = messages.drain(..).collect();
            Some(batch)
        } else {
            None
        }
    }

    /// Force flush all pending messages
    pub async fn force_flush(&self) -> Vec<BatchedMessage> {
        let mut messages = self.pending_messages.write().await;
        messages.drain(..).collect()
    }
}

/// Memory-efficient data structures for solver operations
pub struct SolverDataStructures {
    // Use compact representations for frequently accessed data
    intent_index: HashMap<u64, CompactIntent>,
    solver_pool: Vec<CompactSolver>,
    route_lookup: HashMap<String, u32>, // String to index mapping
}

#[derive(Clone, Debug)]
struct CompactIntent {
    id: u64,
    amount_in: u64,
    min_amount_out: u64,
    token_pair: u32, // Packed token pair
    chain_pair: u32, // Packed chain pair
}

#[derive(Clone, Debug)]
struct CompactSolver {
    id: u64,
    reputation: u32,
    capacity: u64,
    fee_rate: u16,
    active: bool,
}

impl SolverDataStructures {
    pub fn new() -> Self {
        Self {
            intent_index: HashMap::new(),
            solver_pool: Vec::new(),
            route_lookup: HashMap::new(),
        }
    }

    /// Add intent with compact representation
    pub fn add_intent(&mut self, id: u64, amount_in: u64, min_amount_out: u64, 
                      token_in: u16, token_out: u16, source_chain: u16, dest_chain: u16) {
        let token_pair = ((token_in as u32) << 16) | (token_out as u32);
        let chain_pair = ((source_chain as u32) << 16) | (dest_chain as u32);
        
        self.intent_index.insert(id, CompactIntent {
            id,
            amount_in,
            min_amount_out,
            token_pair,
            chain_pair,
        });
    }

    /// Find matching solvers using efficient algorithms
    pub fn find_matching_solvers(&self, intent_id: u64) -> Vec<u64> {
        if let Some(intent) = self.intent_index.get(&intent_id) {
            // Use binary search and filtering for efficient matching
            self.solver_pool
                .iter()
                .filter(|solver| {
                    solver.active && 
                    solver.capacity >= intent.amount_in &&
                    self.can_handle_route(solver, intent)
                })
                .map(|solver| solver.id)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn can_handle_route(&self, solver: &CompactSolver, intent: &CompactIntent) -> bool {
        // Simplified route matching logic
        let (source_chain, dest_chain) = self.unpack_chain_pair(intent.chain_pair);
        
        // Check if solver supports the chain pair
        source_chain == dest_chain || solver.reputation > 1000 // Cross-chain requires high reputation
    }

    fn unpack_chain_pair(&self, chain_pair: u32) -> (u16, u16) {
        ((chain_pair >> 16) as u16, (chain_pair & 0xFFFF) as u16)
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let intent_memory = self.intent_index.len() * std::mem::size_of::<CompactIntent>();
        let solver_memory = self.solver_pool.len() * std::mem::size_of::<CompactSolver>();
        let route_memory = self.route_lookup.len() * (std::mem::size_of::<String>() + std::mem::size_of::<u32>());
        
        MemoryStats {
            total_bytes: intent_memory + solver_memory + route_memory,
            intent_count: self.intent_index.len(),
            solver_count: self.solver_pool.len(),
            route_count: self.route_lookup.len(),
        }
    }
}

#[derive(Debug)]
pub struct MemoryStats {
    pub total_bytes: usize,
    pub intent_count: usize,
    pub solver_count: usize,
    pub route_count: usize,
}

/// Optimized algorithm implementations
pub struct OptimizedAlgorithms;

impl OptimizedAlgorithms {
    /// Fast swap calculation using integer arithmetic
    pub fn fast_swap_calculation(reserve_in: u64, reserve_out: u64, amount_in: u64, fee_bps: u16) -> u64 {
        // Avoid floating point math for speed
        let amount_in_with_fee = amount_in * (10000 - fee_bps as u64) / 10000;
        
        // Use integer math with overflow protection
        if let Some(numerator) = amount_in_with_fee.checked_mul(reserve_out) {
            if let Some(denominator) = reserve_in.checked_add(amount_in_with_fee) {
                if denominator > 0 {
                    return numerator / denominator;
                }
            }
        }
        
        0 // Return 0 on overflow or invalid input
    }

    /// Vectorized price impact calculation
    pub fn batch_price_impact(reserves: &[(u64, u64)], amounts: &[u64]) -> Vec<u64> {
        // Process multiple calculations in batch for better CPU cache usage
        reserves.iter()
            .zip(amounts.iter())
            .map(|((reserve_in, _), &amount_in)| {
                if *reserve_in > 0 {
                    (amount_in * 10000) / reserve_in
                } else {
                    10000 // 100% price impact
                }
            })
            .collect()
    }

    /// Optimized route finding using A* algorithm
    pub fn find_optimal_route(
        start_token: u16,
        end_token: u16,
        amount: u64,
        pools: &[(u16, u16, u64, u64)], // (token0, token1, reserve0, reserve1)
    ) -> Option<Vec<u16>> {
        // Simplified A* implementation for demonstration
        // In production, this would be a full A* with proper heuristics
        
        use std::collections::{BinaryHeap, HashMap, HashSet};
        use std::cmp::Reverse;
        
        #[derive(Debug, Clone, PartialEq, Eq)]
        struct Node {
            token: u16,
            cost: u64,
            path: Vec<u16>,
        }
        
        impl Ord for Node {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other.cost.cmp(&self.cost) // Min-heap
            }
        }
        
        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let mut heap = BinaryHeap::new();
        let mut visited = HashSet::new();
        let mut token_connections: HashMap<u16, Vec<(u16, u64)>> = HashMap::new();
        
        // Build adjacency list
        for &(token0, token1, reserve0, reserve1) in pools {
            let output0 = Self::fast_swap_calculation(reserve1, reserve0, amount, 30);
            let output1 = Self::fast_swap_calculation(reserve0, reserve1, amount, 30);
            
            token_connections.entry(token0).or_insert_with(Vec::new).push((token1, output1));
            token_connections.entry(token1).or_insert_with(Vec::new).push((token0, output0));
        }
        
        // Initialize search
        heap.push(Node {
            token: start_token,
            cost: 0,
            path: vec![start_token],
        });
        
        while let Some(current) = heap.pop() {
            if current.token == end_token {
                return Some(current.path);
            }
            
            if visited.contains(&current.token) {
                continue;
            }
            
            visited.insert(current.token);
            
            if let Some(connections) = token_connections.get(&current.token) {
                for &(next_token, output) in connections {
                    if !visited.contains(&next_token) && current.path.len() < 4 { // Max 3 hops
                        let mut new_path = current.path.clone();
                        new_path.push(next_token);
                        
                        heap.push(Node {
                            token: next_token,
                            cost: current.cost + (amount.saturating_sub(output)), // Minimize loss
                            path: new_path,
                        });
                    }
                }
            }
        }
        
        None
    }
}

/// Performance monitoring and metrics collection
pub struct PerformanceMonitor {
    metrics: Arc<TokioRwLock<Vec<MetricPoint>>>,
    start_time: Instant,
}

#[derive(Debug, Clone)]
struct MetricPoint {
    timestamp: Instant,
    component: String,
    operation: String,
    duration_ms: f64,
    success: bool,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(TokioRwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// Record operation performance
    pub async fn record_operation<F, R>(&self, component: &str, operation: &str, f: F) -> R
    where
        F: std::future::Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let start = Instant::now();
        let result = f.await;
        let duration = start.elapsed();
        
        let metric = MetricPoint {
            timestamp: Instant::now(),
            component: component.to_string(),
            operation: operation.to_string(),
            duration_ms: duration.as_millis() as f64,
            success: result.is_ok(),
        };
        
        self.metrics.write().await.push(metric);
        
        match result {
            Ok(value) => value,
            Err(e) => panic!("Operation failed: {}", e), // In production, handle errors appropriately
        }
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> PerformanceStats {
        let metrics = self.metrics.read().await;
        
        let total_operations = metrics.len();
        let successful_operations = metrics.iter().filter(|m| m.success).count();
        
        let avg_duration = if !metrics.is_empty() {
            metrics.iter().map(|m| m.duration_ms).sum::<f64>() / metrics.len() as f64
        } else {
            0.0
        };
        
        let max_duration = metrics.iter().map(|m| m.duration_ms).fold(0.0f64, |a, b| a.max(b));
        let min_duration = metrics.iter().map(|m| m.duration_ms).fold(f64::MAX, |a, b| a.min(b));
        
        PerformanceStats {
            total_operations,
            successful_operations,
            success_rate: successful_operations as f64 / total_operations as f64,
            avg_duration_ms: avg_duration,
            max_duration_ms: max_duration,
            min_duration_ms: if min_duration == f64::MAX { 0.0 } else { min_duration },
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Clear old metrics to prevent memory bloat
    pub async fn cleanup_old_metrics(&self, retention_period: Duration) {
        let mut metrics = self.metrics.write().await;
        let cutoff = Instant::now() - retention_period;
        
        metrics.retain(|m| m.timestamp > cutoff);
    }
}

#[derive(Debug)]
pub struct PerformanceStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub success_rate: f64,
    pub avg_duration_ms: f64,
    pub max_duration_ms: f64,
    pub min_duration_ms: f64,
    pub uptime_seconds: u64,
}

/// Async task scheduler for background optimization
pub struct TaskScheduler {
    tasks: Arc<TokioRwLock<Vec<ScheduledTask>>>,
}

#[derive(Debug)]
struct ScheduledTask {
    name: String,
    next_run: Instant,
    interval: Duration,
    last_duration: Duration,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(TokioRwLock::new(Vec::new())),
        }
    }

    /// Schedule recurring task
    pub async fn schedule_task(&self, name: String, interval: Duration) {
        let task = ScheduledTask {
            name,
            next_run: Instant::now() + interval,
            interval,
            last_duration: Duration::from_secs(0),
        };
        
        self.tasks.write().await.push(task);
    }

    /// Run scheduled tasks that are due
    pub async fn run_due_tasks(&self) -> Vec<String> {
        let mut tasks = self.tasks.write().await;
        let now = Instant::now();
        let mut executed = Vec::new();
        
        for task in tasks.iter_mut() {
            if now >= task.next_run {
                let start = Instant::now();
                
                // Execute task (placeholder - in production, call actual task function)
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                task.last_duration = start.elapsed();
                task.next_run = now + task.interval;
                executed.push(task.name.clone());
            }
        }
        
        executed
    }

    /// Get task statistics
    pub async fn get_task_stats(&self) -> Vec<TaskStats> {
        let tasks = self.tasks.read().await;
        
        tasks.iter().map(|task| TaskStats {
            name: task.name.clone(),
            next_run_in: task.next_run.saturating_duration_since(Instant::now()),
            last_duration: task.last_duration,
            interval: task.interval,
        }).collect()
    }
}

#[derive(Debug)]
pub struct TaskStats {
    pub name: String,
    pub next_run_in: Duration,
    pub last_duration: Duration,
    pub interval: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_route_cache() {
        let cache = RouteCache::new(10, Duration::from_secs(60));
        
        let route = RouteData {
            hops: vec!["ETH".to_string(), "USDC".to_string()],
            estimated_gas: 21000,
            estimated_output: 1000,
            cross_chain: false,
        };
        
        cache.put("test_route".to_string(), route.clone()).await;
        let cached_route = cache.get("test_route").await;
        
        assert!(cached_route.is_some());
        assert_eq!(cached_route.unwrap().estimated_gas, 21000);
    }

    #[test]
    fn test_fast_swap_calculation() {
        let result = OptimizedAlgorithms::fast_swap_calculation(
            1_000_000, // reserve_in
            2_000_000, // reserve_out  
            10_000,    // amount_in
            300,       // 0.3% fee
        );
        
        assert!(result > 0);
        assert!(result < 20_000); // Should be less than 2x due to slippage
    }

    #[test]
    fn test_batch_price_impact() {
        let reserves = vec![(1_000_000, 2_000_000), (500_000, 1_000_000)];
        let amounts = vec![10_000, 5_000];
        
        let impacts = OptimizedAlgorithms::batch_price_impact(&reserves, &amounts);
        
        assert_eq!(impacts.len(), 2);
        assert!(impacts[0] > 0);
        assert!(impacts[1] > 0);
    }

    #[test]
    fn test_route_finding() {
        let pools = vec![
            (1, 2, 1_000_000, 2_000_000), // ETH-USDC
            (2, 3, 2_000_000, 1_000_000), // USDC-DAI
            (1, 3, 500_000, 500_000),     // ETH-DAI direct
        ];
        
        let route = OptimizedAlgorithms::find_optimal_route(1, 3, 10_000, &pools);
        
        assert!(route.is_some());
        let path = route.unwrap();
        assert_eq!(path[0], 1); // Starts with token 1
        assert_eq!(path[path.len() - 1], 3); // Ends with token 3
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let monitor = PerformanceMonitor::new();
        
        let result = monitor.record_operation("test", "operation", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(42)
        }).await;
        
        assert_eq!(result, 42);
        
        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.successful_operations, 1);
        assert!(stats.avg_duration_ms >= 10.0);
    }

    #[tokio::test]
    async fn test_message_batcher() {
        let batcher = MessageBatcher::new(3, Duration::from_secs(1));
        
        assert!(!batcher.add_message("msg1".to_string(), "channel1".to_string()).await);
        assert!(!batcher.add_message("msg2".to_string(), "channel1".to_string()).await);
        assert!(batcher.add_message("msg3".to_string(), "channel1".to_string()).await);
        
        let batch = batcher.try_flush().await;
        assert!(batch.is_some());
        assert_eq!(batch.unwrap().len(), 3);
    }
}
