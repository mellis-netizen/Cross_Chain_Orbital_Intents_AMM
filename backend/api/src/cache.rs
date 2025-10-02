use redis::{Client, aio::MultiplexedConnection, AsyncCommands, RedisResult};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use crate::{error::Result, models::*};
use ethers::types::{Address, H256};
use chrono::{DateTime, Utc};

// Redis client setup
pub async fn create_client(redis_url: &str) -> Result<MultiplexedConnection> {
    let client = Client::open(redis_url)
        .map_err(|e| crate::error::internal_error(format!("Failed to create Redis client: {}", e)))?;
    
    let connection = client.get_multiplexed_async_connection().await
        .map_err(|e| crate::error::internal_error(format!("Failed to connect to Redis: {}", e)))?;
    
    Ok(connection)
}

// Cache keys
pub struct CacheKeys;

impl CacheKeys {
    pub fn intent_status(intent_id: H256) -> String {
        format!("intent:status:{:#x}", intent_id)
    }
    
    pub fn intent_progress(intent_id: H256) -> String {
        format!("intent:progress:{:#x}", intent_id)
    }
    
    pub fn solver_reputation(address: Address) -> String {
        format!("solver:reputation:{:#x}", address)
    }
    
    pub fn chain_health(chain_id: u64) -> String {
        format!("chain:health:{}", chain_id)
    }
    
    pub fn token_price(chain_id: u64, token: Address) -> String {
        format!("price:{}:{:#x}", chain_id, token)
    }
    
    pub fn rate_limit(identifier: &str) -> String {
        format!("rate_limit:{}", identifier)
    }
    
    pub fn analytics_cache() -> String {
        "analytics:dashboard".to_string()
    }
    
    pub fn solver_performance(address: Address) -> String {
        format!("solver:performance:{:#x}", address)
    }
    
    pub fn pending_intents() -> String {
        "intents:pending".to_string()
    }
}

// Cache service
pub struct CacheService {
    connection: MultiplexedConnection,
}

impl CacheService {
    pub fn new(connection: MultiplexedConnection) -> Self {
        Self { connection }
    }

    // Generic cache operations
    pub async fn set<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let serialized = serde_json::to_string(value)
            .map_err(|e| crate::error::internal_error(format!("Failed to serialize value: {}", e)))?;
        
        if let Some(ttl) = ttl {
            self.connection.set_ex(key, serialized, ttl.as_secs())
                .await
                .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        } else {
            self.connection.set(key, serialized)
                .await
                .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        }
        
        Ok(())
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&mut self, key: &str) -> Result<Option<T>> {
        let value: Option<String> = self.connection.get(key)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        
        match value {
            Some(serialized) => {
                let deserialized = serde_json::from_str(&serialized)
                    .map_err(|e| crate::error::internal_error(format!("Failed to deserialize value: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {
        self.connection.del(key)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        Ok(())
    }

    pub async fn exists(&mut self, key: &str) -> Result<bool> {
        let exists: bool = self.connection.exists(key)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        Ok(exists)
    }

    pub async fn expire(&mut self, key: &str, ttl: Duration) -> Result<()> {
        self.connection.expire(key, ttl.as_secs())
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        Ok(())
    }

    // Intent-specific cache operations
    pub async fn cache_intent_status(
        &mut self,
        intent_id: H256,
        status: &IntentStatusResponse,
    ) -> Result<()> {
        let key = CacheKeys::intent_status(intent_id);
        self.set(&key, status, Some(Duration::from_secs(300))).await // 5 minutes
    }

    pub async fn get_intent_status(
        &mut self,
        intent_id: H256,
    ) -> Result<Option<IntentStatusResponse>> {
        let key = CacheKeys::intent_status(intent_id);
        self.get(&key).await
    }

    pub async fn cache_intent_progress(
        &mut self,
        intent_id: H256,
        progress: &IntentProgress,
    ) -> Result<()> {
        let key = CacheKeys::intent_progress(intent_id);
        self.set(&key, progress, Some(Duration::from_secs(60))).await // 1 minute
    }

    pub async fn get_intent_progress(
        &mut self,
        intent_id: H256,
    ) -> Result<Option<IntentProgress>> {
        let key = CacheKeys::intent_progress(intent_id);
        self.get(&key).await
    }

    // Solver cache operations
    pub async fn cache_solver_reputation(
        &mut self,
        address: Address,
        reputation: f64,
    ) -> Result<()> {
        let key = CacheKeys::solver_reputation(address);
        self.set(&key, &reputation, Some(Duration::from_secs(1800))).await // 30 minutes
    }

    pub async fn get_solver_reputation(
        &mut self,
        address: Address,
    ) -> Result<Option<f64>> {
        let key = CacheKeys::solver_reputation(address);
        self.get(&key).await
    }

    pub async fn cache_solver_performance(
        &mut self,
        address: Address,
        performance: &SolverPerformance,
    ) -> Result<()> {
        let key = CacheKeys::solver_performance(address);
        self.set(&key, performance, Some(Duration::from_secs(3600))).await // 1 hour
    }

    pub async fn get_solver_performance(
        &mut self,
        address: Address,
    ) -> Result<Option<SolverPerformance>> {
        let key = CacheKeys::solver_performance(address);
        self.get(&key).await
    }

    // Chain health cache
    pub async fn cache_chain_health(
        &mut self,
        chain_id: u64,
        health: &ChainHealth,
    ) -> Result<()> {
        let key = CacheKeys::chain_health(chain_id);
        self.set(&key, health, Some(Duration::from_secs(30))).await // 30 seconds
    }

    pub async fn get_chain_health(
        &mut self,
        chain_id: u64,
    ) -> Result<Option<ChainHealth>> {
        let key = CacheKeys::chain_health(chain_id);
        self.get(&key).await
    }

    // Token price cache
    pub async fn cache_token_price(
        &mut self,
        chain_id: u64,
        token: Address,
        price: f64,
    ) -> Result<()> {
        let key = CacheKeys::token_price(chain_id, token);
        self.set(&key, &price, Some(Duration::from_secs(120))).await // 2 minutes
    }

    pub async fn get_token_price(
        &mut self,
        chain_id: u64,
        token: Address,
    ) -> Result<Option<f64>> {
        let key = CacheKeys::token_price(chain_id, token);
        self.get(&key).await
    }

    // Analytics cache
    pub async fn cache_analytics(
        &mut self,
        analytics: &AnalyticsResponse,
    ) -> Result<()> {
        let key = CacheKeys::analytics_cache();
        self.set(&key, analytics, Some(Duration::from_secs(600))).await // 10 minutes
    }

    pub async fn get_analytics(&mut self) -> Result<Option<AnalyticsResponse>> {
        let key = CacheKeys::analytics_cache();
        self.get(&key).await
    }

    // Rate limiting
    pub async fn check_rate_limit(
        &mut self,
        identifier: &str,
        limit: u32,
        window: Duration,
    ) -> Result<RateLimitInfo> {
        let key = CacheKeys::rate_limit(identifier);
        
        // Get current count
        let current_count: Option<u32> = self.get(&key).await?;
        let now = chrono::Utc::now();
        
        match current_count {
            Some(count) if count >= limit => {
                // Rate limit exceeded
                let ttl: i64 = self.connection.ttl(&key)
                    .await
                    .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
                
                Ok(RateLimitInfo {
                    requests_remaining: 0,
                    reset_time: now + chrono::Duration::seconds(ttl),
                    window_size: limit,
                })
            }
            Some(count) => {
                // Increment counter
                let new_count: u32 = self.connection.incr(&key, 1)
                    .await
                    .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
                
                let ttl: i64 = self.connection.ttl(&key)
                    .await
                    .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
                
                Ok(RateLimitInfo {
                    requests_remaining: limit.saturating_sub(new_count),
                    reset_time: now + chrono::Duration::seconds(ttl),
                    window_size: limit,
                })
            }
            None => {
                // First request in window
                self.set(&key, &1u32, Some(window)).await?;
                
                Ok(RateLimitInfo {
                    requests_remaining: limit - 1,
                    reset_time: now + chrono::Duration::from_std(window).unwrap(),
                    window_size: limit,
                })
            }
        }
    }

    // Pending intents cache
    pub async fn cache_pending_intents(
        &mut self,
        intents: &[H256],
    ) -> Result<()> {
        let key = CacheKeys::pending_intents();
        let intent_strings: Vec<String> = intents.iter().map(|id| format!("{:#x}", id)).collect();
        self.set(&key, &intent_strings, Some(Duration::from_secs(60))).await // 1 minute
    }

    pub async fn get_pending_intents(&mut self) -> Result<Option<Vec<H256>>> {
        let key = CacheKeys::pending_intents();
        let intent_strings: Option<Vec<String>> = self.get(&key).await?;
        
        match intent_strings {
            Some(strings) => {
                let intents: Result<Vec<H256>> = strings
                    .iter()
                    .map(|s| crate::database::string_to_h256(s))
                    .collect();
                Ok(Some(intents?))
            }
            None => Ok(None),
        }
    }

    // Health check
    pub async fn health_check(&mut self) -> Result<bool> {
        let test_key = "health_check";
        let test_value = "ping";
        
        self.set(&test_key, &test_value, Some(Duration::from_secs(5))).await?;
        let retrieved: Option<String> = self.get(test_key).await?;
        
        match retrieved {
            Some(value) if value == test_value => {
                self.delete(test_key).await?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    // Publish to Redis Pub/Sub for WebSocket notifications
    pub async fn publish_intent_update(
        &mut self,
        intent_id: H256,
        update: &IntentUpdateMessage,
    ) -> Result<()> {
        let channel = format!("intent_updates:{:#x}", intent_id);
        let message = serde_json::to_string(update)
            .map_err(|e| crate::error::internal_error(format!("Failed to serialize update: {}", e)))?;
        
        self.connection.publish(channel, message)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        
        Ok(())
    }

    pub async fn publish_market_data(
        &mut self,
        data: &MarketDataMessage,
    ) -> Result<()> {
        let channel = "market_data";
        let message = serde_json::to_string(data)
            .map_err(|e| crate::error::internal_error(format!("Failed to serialize market data: {}", e)))?;
        
        self.connection.publish(channel, message)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        
        Ok(())
    }

    // Batch operations for performance
    pub async fn batch_cache_solver_reputations(
        &mut self,
        reputations: &[(Address, f64)],
    ) -> Result<()> {
        let mut pipe = redis::pipe();
        
        for (address, reputation) in reputations {
            let key = CacheKeys::solver_reputation(*address);
            let serialized = serde_json::to_string(reputation)
                .map_err(|e| crate::error::internal_error(format!("Failed to serialize reputation: {}", e)))?;
            
            pipe.set_ex(&key, serialized, 1800); // 30 minutes
        }
        
        pipe.query_async(&mut self.connection)
            .await
            .map_err(|e| crate::error::ApiError::Redis(e.to_string()))?;
        
        Ok(())
    }

    // Cache warming
    pub async fn warm_cache(&mut self) -> Result<()> {
        // This can be called on startup to pre-populate frequently accessed data
        tracing::info!("Warming Redis cache...");
        
        // Add cache warming logic here as needed
        // For example, pre-load active solvers, chain health, etc.
        
        Ok(())
    }
}