use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State, Query},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use ethers::types::{Address, H256};

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
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcasters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_connection(&self, conn_info: ConnectionInfo) {
        let mut connections = self.connections.write().await;
        connections.insert(conn_info.id, conn_info);
    }
    
    pub async fn remove_connection(&self, conn_id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&conn_id);
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
    
    // Create connection info
    let conn_info = ConnectionInfo {
        id: conn_id,
        user_address,
        subscriptions: subscriptions.clone(),
        connected_at: Utc::now(),
        last_activity: Utc::now(),
    };
    
    // Add to manager
    WS_MANAGER.add_connection(conn_info).await;
    
    // Split socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    
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
    
    // Spawn task to handle broadcast messages
    let outgoing_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            tokio::select! {
                // Handle broadcast messages
                result = async {
                    for (channel, receiver) in &mut broadcast_receivers {
                        match receiver.try_recv() {
                            Ok(message) => {
                                if let Ok(json) = serde_json::to_string(&message) {
                                    if sender.send(Message::Text(json)).await.is_err() {
                                        return Err("Failed to send message");
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
                
                // Periodic ping
                _ = interval.tick() => {
                    if sender.send(Message::Ping(vec![])).await.is_err() {
                        break;
                    }
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
    ws_manager.remove_connection(conn_id).await;
    tracing::info!("WebSocket {} disconnected", conn_id);
}

// Handle incoming WebSocket messages
async fn handle_incoming_message(
    text: &str,
    _conn_id: Uuid,
    _user_address: Option<Address>,
) -> Result<()> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(json) => {
            if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                match msg_type {
                    "ping" => {
                        // Handle ping - response is handled automatically
                        Ok(())
                    }
                    "subscribe" => {
                        // Handle dynamic subscription changes
                        // TODO: Implement dynamic subscription management
                        Ok(())
                    }
                    "unsubscribe" => {
                        // Handle unsubscription
                        // TODO: Implement dynamic unsubscription
                        Ok(())
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
    serde_json::json!({
        "active_connections": WS_MANAGER.get_connection_count().await,
        "active_subscriptions": WS_MANAGER.get_active_subscriptions().await
    })
}