use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State, Query},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock, mpsc};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use ethers::types::{Address, H256};
use std::time::Duration;
use tokio::time::Instant;
use std::collections::VecDeque;

use crate::{
    models::{AppState, WebSocketMessage, IntentUpdateMessage, MarketDataMessage},
    error::Result,
    auth::validate_jwt,
};

// WebSocket connection parameters
#[derive(Debug, Deserialize)]
pub struct WsParams {
    token: Option<String>,
    subscribe: Option<String>, // Comma-separated list of channels
}

// WebSocket connection info
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub user_address: Option<Address>,
    pub subscriptions: Vec<String>,
    pub connected_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub sender: Option<mpsc::UnboundedSender<Message>>,
    pub health: ConnectionHealth,
}

#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub last_ping: Instant,
    pub last_pong: Instant,
    pub message_count: u64,
    pub error_count: u64,
    pub is_healthy: bool,
    pub rate_limiter: RateLimiter,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    pub messages: VecDeque<Instant>,
    pub max_messages: usize,
    pub window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_messages: usize, window_duration: Duration) -> Self {
        Self {
            messages: VecDeque::new(),
            max_messages,
            window_duration,
        }
    }
    
    pub fn check_rate_limit(&mut self) -> bool {
        let now = Instant::now();
        
        // Remove old messages outside the window
        while let Some(&front) = self.messages.front() {
            if now.duration_since(front) > self.window_duration {
                self.messages.pop_front();
            } else {
                break;
            }
        }
        
        // Check if we're within rate limits
        if self.messages.len() >= self.max_messages {
            return false;
        }
        
        // Add current message
        self.messages.push_back(now);
        true
    }
}

#[derive(Debug, Clone)]
pub struct SubscriptionLimits {
    pub max_subscriptions_per_connection: usize,
    pub max_intent_subscriptions: usize,
    pub max_user_subscriptions: usize,
    pub allowed_patterns: Vec<String>,
}

// WebSocket subscription channels
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscriptionChannel {
    IntentUpdates(H256),     // intent-specific updates
    UserIntents(Address),    // all intents for a user
    MarketData,              // general market data
    SolverUpdates(Address),  // solver-specific updates
    SystemAlerts,            // system-wide alerts
}

impl SubscriptionChannel {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "market_data" => Some(Self::MarketData),
            "system_alerts" => Some(Self::SystemAlerts),
            _ => {
                if let Some(intent_id) = s.strip_prefix("intent:") {
                    if let Ok(id) = intent_id.parse::<H256>() {
                        return Some(Self::IntentUpdates(id));
                    }
                }
                if let Some(user_addr) = s.strip_prefix("user:") {
                    if let Ok(addr) = user_addr.parse::<Address>() {
                        return Some(Self::UserIntents(addr));
                    }
                }
                if let Some(solver_addr) = s.strip_prefix("solver:") {
                    if let Ok(addr) = solver_addr.parse::<Address>() {
                        return Some(Self::SolverUpdates(addr));
                    }
                }
                None
            }
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Self::IntentUpdates(id) => format!("intent:{:#x}", id),
            Self::UserIntents(addr) => format!("user:{:#x}", addr),
            Self::MarketData => "market_data".to_string(),
            Self::SolverUpdates(addr) => format!("solver:{:#x}", addr),
            Self::SystemAlerts => "system_alerts".to_string(),
        }
    }
}

// WebSocket manager
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, ConnectionInfo>>>,
    broadcasters: Arc<RwLock<HashMap<SubscriptionChannel, broadcast::Sender<WebSocketMessage>>>>,
    health_monitor: Arc<RwLock<HashMap<Uuid, Instant>>>,
    metrics: Arc<RwLock<WebSocketMetrics>>,
    subscription_limits: SubscriptionLimits,
}

#[derive(Debug, Default)]
pub struct WebSocketMetrics {
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub subscription_changes: u64,
    pub health_checks: u64,
    pub errors: u64,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcasters: Arc::new(RwLock::new(HashMap::new())),
            health_monitor: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(WebSocketMetrics::default())),
            subscription_limits: SubscriptionLimits {
                max_subscriptions_per_connection: 50,
                max_intent_subscriptions: 20,
                max_user_subscriptions: 10,
                allowed_patterns: vec![
                    "market_data".to_string(),
                    "system_alerts".to_string(),
                    "intent:*".to_string(),
                    "user:*".to_string(),
                    "solver:*".to_string(),
                ],
            },
        }
    }
    
    pub async fn add_connection(&self, conn_info: ConnectionInfo) {
        let mut connections = self.connections.write().await;
        let mut health_monitor = self.health_monitor.write().await;
        let mut metrics = self.metrics.write().await;
        
        connections.insert(conn_info.id, conn_info);
        health_monitor.insert(conn_info.id, Instant::now());
        
        metrics.total_connections += 1;
        metrics.active_connections = connections.len() as u64;
    }
    
    pub async fn remove_connection(&self, conn_id: Uuid) {
        let mut connections = self.connections.write().await;
        let mut health_monitor = self.health_monitor.write().await;
        let mut metrics = self.metrics.write().await;
        
        connections.remove(&conn_id);
        health_monitor.remove(&conn_id);
        
        metrics.active_connections = connections.len() as u64;
    }
    
    pub async fn get_broadcaster(&self, channel: &SubscriptionChannel) -> broadcast::Sender<WebSocketMessage> {
        let mut broadcasters = self.broadcasters.write().await;
        
        if let Some(sender) = broadcasters.get(channel) {
            sender.clone()
        } else {
            let (sender, _) = broadcast::channel(1000);
            broadcasters.insert(channel.clone(), sender.clone());
            sender
        }
    }
    
    pub async fn broadcast_to_channel(&self, channel: SubscriptionChannel, message: WebSocketMessage) {
        let broadcaster = self.get_broadcaster(&channel).await;
        
        if let Err(e) = broadcaster.send(message) {
            tracing::warn!("Failed to broadcast message to channel {:?}: {}", channel, e);
        }
    }
    
    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
    
    pub async fn get_active_subscriptions(&self) -> HashMap<String, usize> {
        let connections = self.connections.read().await;
        let mut subscription_counts = HashMap::new();
        
        for conn in connections.values() {
            for sub in &conn.subscriptions {
                *subscription_counts.entry(sub.clone()).or_insert(0) += 1;
            }
        }
        
        subscription_counts
    }
    
    /// Subscribe a connection to a channel with enhanced validation
    pub async fn subscribe_to_channel(
        &self,
        conn_id: Uuid,
        channel: SubscriptionChannel,
    ) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(conn) = connections.get_mut(&conn_id) {
            let channel_str = channel.to_string();
            
            // Check if already subscribed
            if conn.subscriptions.contains(&channel_str) {
                return Ok(()); // Already subscribed
            }
            
            // Check subscription limits
            if conn.subscriptions.len() >= self.subscription_limits.max_subscriptions_per_connection {
                return Err(crate::error::ApiError::BadRequest(
                    format!("Subscription limit exceeded: max {} per connection", 
                            self.subscription_limits.max_subscriptions_per_connection)
                ));
            }
            
            // Check specific channel type limits
            match &channel {
                SubscriptionChannel::IntentUpdates(_) => {
                    let intent_subs = conn.subscriptions.iter()
                        .filter(|s| s.starts_with("intent:"))
                        .count();
                    if intent_subs >= self.subscription_limits.max_intent_subscriptions {
                        return Err(crate::error::ApiError::BadRequest(
                            format!("Intent subscription limit exceeded: max {}", 
                                    self.subscription_limits.max_intent_subscriptions)
                        ));
                    }
                }
                SubscriptionChannel::UserIntents(_) | SubscriptionChannel::SolverUpdates(_) => {
                    let user_subs = conn.subscriptions.iter()
                        .filter(|s| s.starts_with("user:") || s.starts_with("solver:"))
                        .count();
                    if user_subs >= self.subscription_limits.max_user_subscriptions {
                        return Err(crate::error::ApiError::BadRequest(
                            format!("User/Solver subscription limit exceeded: max {}", 
                                    self.subscription_limits.max_user_subscriptions)
                        ));
                    }
                }
                _ => {} // No specific limits for other channels
            }
            
            // Add to connection's subscriptions
            conn.subscriptions.push(channel_str);
            conn.last_activity = Utc::now();
            
            // Ensure broadcaster exists for this channel
            self.get_broadcaster(&channel).await;
            
            tracing::debug!("Connection {} subscribed to channel: {:?}", conn_id, channel);
            Ok(())
        } else {
            Err(crate::error::ApiError::NotFound("Connection not found".to_string()))
        }
    }
    
    /// Unsubscribe a connection from a channel
    pub async fn unsubscribe_from_channel(
        &self,
        conn_id: Uuid,
        channel: SubscriptionChannel,
    ) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(conn) = connections.get_mut(&conn_id) {
            let channel_str = channel.to_string();
            
            // Remove from connection's subscriptions
            conn.subscriptions.retain(|s| s != &channel_str);
            conn.last_activity = Utc::now();
            
            tracing::debug!("Connection {} unsubscribed from channel: {:?}", conn_id, channel);
            Ok(())
        } else {
            Err(crate::error::ApiError::NotFound("Connection not found".to_string()))
        }
    }
    
    /// Send a message to a specific connection
    pub async fn send_to_connection(
        &self,
        conn_id: Uuid,
        message: serde_json::Value,
    ) -> Result<()> {
        let connections = self.connections.read().await;
        let mut metrics = self.metrics.write().await;
        
        if let Some(conn) = connections.get(&conn_id) {
            if let Some(sender) = &conn.sender {
                let message_text = serde_json::to_string(&message)
                    .map_err(|e| crate::error::validation_error(format!("Failed to serialize message: {}", e)))?;
                
                if let Err(_) = sender.send(Message::Text(message_text)) {
                    // Connection is closed, it will be cleaned up by the health monitor
                    tracing::warn!("Failed to send message to connection {}: sender closed", conn_id);
                    metrics.errors += 1;
                    return Err(crate::error::ApiError::Internal("Connection closed".to_string()));
                }
                
                metrics.messages_sent += 1;
                Ok(())
            } else {
                Err(crate::error::ApiError::Internal("Connection has no sender".to_string()))
            }
        } else {
            Err(crate::error::ApiError::NotFound("Connection not found".to_string()))
        }
    }
    
    /// Update connection activity timestamp
    pub async fn update_activity(&self, conn_id: Uuid) {
        let mut connections = self.connections.write().await;
        
        if let Some(conn) = connections.get_mut(&conn_id) {
            conn.last_activity = Utc::now();
        }
    }
    
    /// Get connections that are subscribed to a specific channel
    pub async fn get_subscribers(&self, channel: &SubscriptionChannel) -> Vec<Uuid> {
        let connections = self.connections.read().await;
        let channel_str = channel.to_string();
        
        connections
            .iter()
            .filter(|(_, conn)| conn.subscriptions.contains(&channel_str))
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// Clean up inactive connections (older than 5 minutes)
    pub async fn cleanup_inactive_connections(&self) {
        let mut connections = self.connections.write().await;
        let mut health_monitor = self.health_monitor.write().await;
        let cutoff_time = Utc::now() - chrono::Duration::minutes(5);
        
        let inactive_connections: Vec<Uuid> = connections
            .iter()
            .filter(|(_, conn)| conn.last_activity < cutoff_time)
            .map(|(id, _)| *id)
            .collect();
        
        for conn_id in inactive_connections {
            connections.remove(&conn_id);
            health_monitor.remove(&conn_id);
            tracing::info!("Cleaned up inactive connection: {}", conn_id);
        }
    }
    
    /// Monitor connection health and perform health checks
    pub async fn perform_health_checks(&self) {
        let mut connections = self.connections.write().await;
        let mut health_monitor = self.health_monitor.write().await;
        let mut metrics = self.metrics.write().await;
        
        let now = Instant::now();
        let health_timeout = Duration::from_secs(60); // 1 minute timeout
        
        let mut unhealthy_connections = Vec::new();
        
        for (conn_id, last_health_check) in health_monitor.iter() {
            if now.duration_since(*last_health_check) > health_timeout {
                if let Some(conn) = connections.get_mut(conn_id) {
                    conn.health.is_healthy = false;
                    conn.health.error_count += 1;
                    
                    // If connection has too many errors, mark for removal
                    if conn.health.error_count > 5 {
                        unhealthy_connections.push(*conn_id);
                    } else if let Some(sender) = &conn.sender {
                        // Send ping to check if connection is alive
                        if sender.send(Message::Ping(vec![])).is_err() {
                            unhealthy_connections.push(*conn_id);
                        }
                    }
                }
            }
        }
        
        // Remove unhealthy connections
        for conn_id in unhealthy_connections {
            connections.remove(&conn_id);
            health_monitor.remove(&conn_id);
            tracing::info!("Removed unhealthy connection: {}", conn_id);
        }
        
        metrics.health_checks += 1;
        metrics.active_connections = connections.len() as u64;
    }
    
    /// Update connection health status
    pub async fn update_connection_health(&self, conn_id: Uuid, is_healthy: bool) {
        let mut connections = self.connections.write().await;
        let mut health_monitor = self.health_monitor.write().await;
        
        if let Some(conn) = connections.get_mut(&conn_id) {
            conn.health.is_healthy = is_healthy;
            if is_healthy {
                conn.health.last_pong = Instant::now();
                conn.health.error_count = 0;
            } else {
                conn.health.error_count += 1;
            }
            health_monitor.insert(conn_id, Instant::now());
        }
    }
    
    /// Get comprehensive WebSocket metrics
    pub async fn get_comprehensive_metrics(&self) -> serde_json::Value {
        let connections = self.connections.read().await;
        let metrics = self.metrics.read().await;
        
        let healthy_connections = connections
            .values()
            .filter(|conn| conn.health.is_healthy)
            .count();
        
        let subscription_stats = self.get_active_subscriptions().await;
        
        serde_json::json!({
            "active_connections": metrics.active_connections,
            "total_connections": metrics.total_connections,
            "healthy_connections": healthy_connections,
            "unhealthy_connections": metrics.active_connections - healthy_connections as u64,
            "messages_sent": metrics.messages_sent,
            "messages_received": metrics.messages_received,
            "subscription_changes": metrics.subscription_changes,
            "health_checks": metrics.health_checks,
            "errors": metrics.errors,
            "subscription_stats": subscription_stats,
            "timestamp": Utc::now()
        })
    }
}

// Global WebSocket manager instance
lazy_static::lazy_static! {
    pub static ref WS_MANAGER: WebSocketManager = WebSocketManager::new();
}

// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<WsParams>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, state, params))
}

// Handle individual WebSocket connection
async fn handle_websocket(
    socket: WebSocket,
    state: AppState,
    params: WsParams,
) {
    let conn_id = Uuid::new_v4();
    let mut user_address = None;
    let mut subscriptions = Vec::new();
    
    // Authenticate if token provided
    if let Some(token) = &params.token {
        match validate_jwt(token, &state.config.jwt_secret) {
            Ok(claims) => {
                if let Ok(addr) = claims.sub.parse::<Address>() {
                    user_address = Some(addr);
                    tracing::info!("WebSocket authenticated for user: {:#x}", addr);
                } else {
                    tracing::warn!("Invalid address in JWT claims");
                }
            }
            Err(e) => {
                tracing::warn!("WebSocket authentication failed: {}", e);
                return; // Close connection
            }
        }
    }
    
    // Parse subscriptions
    if let Some(subscribe_str) = &params.subscribe {
        for channel_str in subscribe_str.split(',') {
            let channel_str = channel_str.trim();
            if !channel_str.is_empty() {
                subscriptions.push(channel_str.to_string());
            }
        }
    }
    
    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    
    // Create a channel for sending messages to this connection
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    
    // Create connection info with sender
    let conn_info = ConnectionInfo {
        id: conn_id,
        user_address,
        subscriptions: subscriptions.clone(),
        connected_at: Utc::now(),
        last_activity: Utc::now(),
        sender: Some(tx),
        health: ConnectionHealth {
            last_ping: Instant::now(),
            last_pong: Instant::now(),
            message_count: 0,
            error_count: 0,
            is_healthy: true,
            rate_limiter: RateLimiter::new(60, Duration::from_secs(60)), // 60 messages per minute
        },
    };
    
    // Add to manager
    WS_MANAGER.add_connection(conn_info).await;
    
    // Create broadcast receivers for subscribed channels
    let mut broadcast_receivers = Vec::new();
    
    for sub in &subscriptions {
        if let Some(channel) = SubscriptionChannel::from_string(sub) {
            // Check permissions
            if can_subscribe_to_channel(&channel, user_address) {
                let broadcaster = WS_MANAGER.get_broadcaster(&channel).await;
                let receiver = broadcaster.subscribe();
                broadcast_receivers.push((channel, receiver));
                
                tracing::info!("WebSocket {} subscribed to channel: {}", conn_id, sub);
            } else {
                tracing::warn!("WebSocket {} denied subscription to channel: {}", conn_id, sub);
            }
        }
    }
    
    // Send welcome message
    let welcome_msg = WebSocketMessage {
        message_type: "welcome".to_string(),
        data: serde_json::json!({
            "connection_id": conn_id,
            "subscriptions": subscriptions,
            "authenticated": user_address.is_some()
        }),
        timestamp: Utc::now(),
    };
    
    if let Ok(welcome_json) = serde_json::to_string(&welcome_msg) {
        if sender.send(Message::Text(welcome_json)).await.is_err() {
            return;
        }
    }
    
    // Spawn task to handle incoming messages
    let ws_manager = &WS_MANAGER;
    let conn_id_clone = conn_id;
    
    let incoming_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = handle_incoming_message(&text, conn_id_clone, user_address).await {
                        tracing::warn!("Error handling WebSocket message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket {} closed by client", conn_id_clone);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    // Echo pong
                    if sender.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Update last activity
                }
                Err(e) => {
                    tracing::warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {} // Binary messages not supported
            }
        }
    });
    
    // Spawn task to handle outgoing messages (both direct and broadcast)
    let outgoing_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            tokio::select! {
                // Handle direct messages from mpsc channel
                Some(message) = rx.recv() => {
                    if sender.send(message).await.is_err() {
                        tracing::warn!("Failed to send direct message to connection {}", conn_id);
                        break;
                    }
                }
                
                // Handle broadcast messages
                result = async {
                    for (channel, receiver) in &mut broadcast_receivers {
                        match receiver.try_recv() {
                            Ok(message) => {
                                if let Ok(json) = serde_json::to_string(&message) {
                                    if sender.send(Message::Text(json)).await.is_err() {
                                        return Err("Failed to send broadcast message");
                                    }
                                }
                            }
                            Err(broadcast::error::TryRecvError::Empty) => {
                                // No message, continue
                            }
                            Err(broadcast::error::TryRecvError::Lagged(n)) => {
                                tracing::warn!("WebSocket {} lagged by {} messages on channel {:?}", conn_id, n, channel);
                            }
                            Err(broadcast::error::TryRecvError::Closed) => {
                                tracing::info!("Broadcast channel {:?} closed", channel);
                                return Err("Channel closed");
                            }
                        }
                    }
                    Ok(())
                } => {
                    if result.is_err() {
                        break;
                    }
                }
                
                // Periodic ping for health monitoring
                _ = interval.tick() => {
                    if sender.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
                    WS_MANAGER.update_connection_health(conn_id, true).await;
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = incoming_task => {},
        _ = outgoing_task => {},
    }
    
    // Clean up
    WS_MANAGER.remove_connection(conn_id).await;
    tracing::info!("WebSocket {} disconnected", conn_id);
}

// Handle incoming WebSocket messages
async fn handle_incoming_message(
    text: &str,
    conn_id: Uuid,
    user_address: Option<Address>,
) -> Result<()> {
    // Check rate limits first
    {
        let mut connections = WS_MANAGER.connections.write().await;
        if let Some(conn) = connections.get_mut(&conn_id) {
            if !conn.health.rate_limiter.check_rate_limit() {
                // Rate limit exceeded
                WS_MANAGER.update_connection_health(conn_id, false).await;
                return Err(crate::error::ApiError::RateLimit);
            }
            conn.health.message_count += 1;
        }
    }
    
    // Update message metrics
    {
        let mut metrics = WS_MANAGER.metrics.write().await;
        metrics.messages_received += 1;
    }
    
    // Update last activity
    WS_MANAGER.update_activity(conn_id).await;
    
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(json) => {
            if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                match msg_type {
                    "ping" => {
                        // Handle ping - update health and respond with pong
                        WS_MANAGER.update_connection_health(conn_id, true).await;
                        Ok(())
                    }
                    "subscribe" => {
                        // Handle dynamic subscription changes
                        handle_subscribe_message(
                            &WS_MANAGER,
                            conn_id,
                            user_address,
                            &json
                        ).await
                    }
                    "unsubscribe" => {
                        // Handle unsubscription
                        handle_unsubscribe_message(
                            &WS_MANAGER,
                            conn_id,
                            &json
                        ).await
                    }
                    _ => {
                        tracing::warn!("Unknown WebSocket message type: {}", msg_type);
                        Ok(())
                    }
                }
            } else {
                Err(crate::error::validation_error("Missing message type"))
            }
        }
        Err(e) => {
            // Update error metrics
            {
                let mut metrics = WS_MANAGER.metrics.write().await;
                metrics.errors += 1;
            }
            WS_MANAGER.update_connection_health(conn_id, false).await;
            Err(crate::error::validation_error(format!("Invalid JSON: {}", e)))
        }
    }
}

// Check if user can subscribe to a channel
fn can_subscribe_to_channel(
    channel: &SubscriptionChannel,
    user_address: Option<Address>,
) -> bool {
    match channel {
        SubscriptionChannel::MarketData | SubscriptionChannel::SystemAlerts => {
            // Public channels
            true
        }
        SubscriptionChannel::UserIntents(addr) | SubscriptionChannel::SolverUpdates(addr) => {
            // User can only subscribe to their own data
            user_address == Some(*addr)
        }
        SubscriptionChannel::IntentUpdates(_) => {
            // Intent updates require authentication
            user_address.is_some()
        }
    }
}

// Helper functions to broadcast updates
pub async fn broadcast_intent_update(
    intent_id: H256,
    update: IntentUpdateMessage,
) {
    let message = WebSocketMessage {
        message_type: "intent_update".to_string(),
        data: serde_json::to_value(&update).unwrap_or_default(),
        timestamp: Utc::now(),
    };
    
    // Broadcast to intent-specific channel
    WS_MANAGER.broadcast_to_channel(
        SubscriptionChannel::IntentUpdates(intent_id),
        message.clone()
    ).await;
    
    // Also broadcast to user's channel if we can determine the user
    // This would require looking up the intent in the database
}

pub async fn broadcast_market_data(
    data: MarketDataMessage,
) {
    let message = WebSocketMessage {
        message_type: "market_data".to_string(),
        data: serde_json::to_value(&data).unwrap_or_default(),
        timestamp: Utc::now(),
    };
    
    WS_MANAGER.broadcast_to_channel(
        SubscriptionChannel::MarketData,
        message
    ).await;
}

pub async fn broadcast_system_alert(
    alert_type: &str,
    message: &str,
    severity: &str,
) {
    let alert_data = serde_json::json!({
        "alert_type": alert_type,
        "message": message,
        "severity": severity
    });
    
    let ws_message = WebSocketMessage {
        message_type: "system_alert".to_string(),
        data: alert_data,
        timestamp: Utc::now(),
    };
    
    WS_MANAGER.broadcast_to_channel(
        SubscriptionChannel::SystemAlerts,
        ws_message
    ).await;
}

// WebSocket metrics
pub async fn get_websocket_metrics() -> serde_json::Value {
    WS_MANAGER.get_comprehensive_metrics().await
}

// Start health monitoring background task
pub async fn start_health_monitoring() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Perform health checks
            WS_MANAGER.perform_health_checks().await;
            
            // Clean up inactive connections
            WS_MANAGER.cleanup_inactive_connections().await;
        }
    });
}

// Dynamic subscription handlers

/// Handle subscribe message for dynamic subscription management
async fn handle_subscribe_message(
    ws_manager: &WebSocketManager,
    connection_id: Uuid,
    user_address: Option<Address>,
    msg: &serde_json::Value,
) -> Result<()> {
    #[derive(Deserialize)]
    struct SubscribeRequest {
        channels: Vec<String>,
    }
    
    // Parse the subscription request
    let sub_request: SubscribeRequest = serde_json::from_value(msg.clone())
        .map_err(|e| crate::error::validation_error(format!("Invalid subscribe request: {}", e)))?;
    
    if sub_request.channels.is_empty() {
        return Err(crate::error::validation_error("No channels specified"));
    }
    
    if sub_request.channels.len() > 20 {
        return Err(crate::error::validation_error("Too many channels (max 20)"));
    }
    
    let mut subscribed_channels = Vec::new();
    let mut failed_channels = Vec::new();
    
    for channel_str in sub_request.channels {
        if let Some(channel) = SubscriptionChannel::from_string(&channel_str) {
            // Check authorization
            if can_subscribe_to_channel(&channel, user_address) {
                // Subscribe to the channel
                match ws_manager.subscribe_to_channel(connection_id, channel.clone()).await {
                    Ok(_) => {
                        subscribed_channels.push(channel_str.clone());
                        tracing::info!(
                            "Connection {} subscribed to channel: {}",
                            connection_id,
                            channel_str
                        );
                        
                        // Update subscription metrics
                        {
                            let mut metrics = ws_manager.metrics.write().await;
                            metrics.subscription_changes += 1;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to subscribe connection {} to channel {}: {}",
                            connection_id,
                            channel_str,
                            e
                        );
                        failed_channels.push((channel_str, format!("Subscription failed: {}", e)));
                        
                        // Update error metrics
                        {
                            let mut metrics = ws_manager.metrics.write().await;
                            metrics.errors += 1;
                        }
                    }
                }
            } else {
                failed_channels.push((channel_str, "Access denied".to_string()));
            }
        } else {
            failed_channels.push((channel_str, "Invalid channel format".to_string()));
        }
    }
    
    // Send confirmation back to client
    let response = serde_json::json!({
        "type": "subscribe_response",
        "success": !subscribed_channels.is_empty(),
        "subscribed": subscribed_channels,
        "failed": failed_channels,
        "timestamp": Utc::now()
    });
    
    ws_manager.send_to_connection(connection_id, response).await?;
    
    Ok(())
}

/// Handle unsubscribe message for dynamic unsubscription
async fn handle_unsubscribe_message(
    ws_manager: &WebSocketManager,
    connection_id: Uuid,
    msg: &serde_json::Value,
) -> Result<()> {
    #[derive(Deserialize)]
    struct UnsubscribeRequest {
        channels: Vec<String>,
    }
    
    // Parse the unsubscription request
    let unsub_request: UnsubscribeRequest = serde_json::from_value(msg.clone())
        .map_err(|e| crate::error::validation_error(format!("Invalid unsubscribe request: {}", e)))?;
    
    if unsub_request.channels.is_empty() {
        return Err(crate::error::validation_error("No channels specified"));
    }
    
    let mut unsubscribed_channels = Vec::new();
    let mut failed_channels = Vec::new();
    
    for channel_str in unsub_request.channels {
        if let Some(channel) = SubscriptionChannel::from_string(&channel_str) {
            match ws_manager.unsubscribe_from_channel(connection_id, channel).await {
                Ok(_) => {
                    unsubscribed_channels.push(channel_str.clone());
                    tracing::info!(
                        "Connection {} unsubscribed from channel: {}",
                        connection_id,
                        channel_str
                    );
                    
                    // Update subscription metrics
                    {
                        let mut metrics = ws_manager.metrics.write().await;
                        metrics.subscription_changes += 1;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to unsubscribe connection {} from channel {}: {}",
                        connection_id,
                        channel_str,
                        e
                    );
                    failed_channels.push((channel_str, format!("Unsubscription failed: {}", e)));
                    
                    // Update error metrics
                    {
                        let mut metrics = ws_manager.metrics.write().await;
                        metrics.errors += 1;
                    }
                }
            }
        } else {
            failed_channels.push((channel_str, "Invalid channel format".to_string()));
        }
    }
    
    // Send confirmation back to client
    let response = serde_json::json!({
        "type": "unsubscribe_response",
        "success": !unsubscribed_channels.is_empty(),
        "unsubscribed": unsubscribed_channels,
        "failed": failed_channels,
        "timestamp": Utc::now()
    });
    
    ws_manager.send_to_connection(connection_id, response).await?;
    
    Ok(())
}