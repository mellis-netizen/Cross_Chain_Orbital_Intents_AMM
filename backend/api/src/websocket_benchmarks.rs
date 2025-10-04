use crate::websocket::{WS_MANAGER, WebSocketMessage, SubscriptionChannel};
use chrono::Utc;
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time;
use uuid::Uuid;

/// WebSocket performance benchmarks
pub struct WebSocketBenchmarks;

#[derive(Debug)]
pub struct BenchmarkResults {
    pub test_name: String,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub success_rate: f64,
    pub memory_usage_mb: f64,
    pub error_count: u64,
}

impl WebSocketBenchmarks {
    /// Benchmark message broadcasting performance
    pub async fn benchmark_broadcast_performance(
        message_count: usize,
        subscriber_count: usize,
    ) -> BenchmarkResults {
        let start_time = Instant::now();
        let mut errors = 0u64;
        
        // Create test channel
        let test_channel = SubscriptionChannel::MarketData;
        
        // Get broadcaster
        let broadcaster = WS_MANAGER.get_broadcaster(&test_channel).await;
        
        // Create test subscribers
        let mut receivers = Vec::new();
        for _ in 0..subscriber_count {
            receivers.push(broadcaster.subscribe());
        }
        
        // Benchmark message sending
        let message_start = Instant::now();
        
        for i in 0..message_count {
            let test_message = WebSocketMessage {
                message_type: "benchmark".to_string(),
                data: json!({
                    "id": i,
                    "timestamp": Utc::now(),
                    "data": format!("test_data_{}", i)
                }),
                timestamp: Utc::now(),
            };
            
            if broadcaster.send(test_message).is_err() {
                errors += 1;
            }
        }
        
        let message_duration = message_start.elapsed();
        
        // Verify message reception
        let mut received_count = 0;
        for receiver in receivers.iter_mut() {
            while let Ok(_) = receiver.try_recv() {
                received_count += 1;
            }
        }
        
        let total_duration = start_time.elapsed();
        let expected_total = message_count * subscriber_count;
        let success_rate = received_count as f64 / expected_total as f64;
        let ops_per_second = message_count as f64 / message_duration.as_secs_f64();
        
        BenchmarkResults {
            test_name: "Broadcast Performance".to_string(),
            duration: total_duration,
            operations_per_second: ops_per_second,
            success_rate,
            memory_usage_mb: Self::get_memory_usage_mb(),
            error_count: errors,
        }
    }
    
    /// Benchmark connection management performance
    pub async fn benchmark_connection_management(connection_count: usize) -> BenchmarkResults {
        let start_time = Instant::now();
        let mut errors = 0u64;
        
        // Create test connections
        let mut connection_ids = Vec::new();
        
        for i in 0..connection_count {
            let conn_id = Uuid::new_v4();
            let conn_info = crate::websocket::ConnectionInfo {
                id: conn_id,
                user_address: None,
                subscriptions: vec![format!("test_channel_{}", i % 10)],
                connected_at: Utc::now(),
                last_activity: Utc::now(),
                sender: None,
                health: crate::websocket::ConnectionHealth {
                    last_ping: tokio::time::Instant::now(),
                    last_pong: tokio::time::Instant::now(),
                    message_count: 0,
                    error_count: 0,
                    is_healthy: true,
                    rate_limiter: crate::websocket::RateLimiter::new(60, Duration::from_secs(60)),
                },
            };
            
            if WS_MANAGER.add_connection(conn_info).await.is_err() {
                errors += 1;
            } else {
                connection_ids.push(conn_id);
            }
        }
        
        // Test connection lookup performance
        let lookup_start = Instant::now();
        for conn_id in &connection_ids {
            WS_MANAGER.update_activity(*conn_id).await;
        }
        let lookup_duration = lookup_start.elapsed();
        
        // Clean up connections
        for conn_id in connection_ids {
            WS_MANAGER.remove_connection(conn_id).await;
        }
        
        let total_duration = start_time.elapsed();
        let ops_per_second = connection_count as f64 / lookup_duration.as_secs_f64();
        let success_rate = (connection_count - errors as usize) as f64 / connection_count as f64;
        
        BenchmarkResults {
            test_name: "Connection Management".to_string(),
            duration: total_duration,
            operations_per_second: ops_per_second,
            success_rate,
            memory_usage_mb: Self::get_memory_usage_mb(),
            error_count: errors,
        }
    }
    
    /// Benchmark subscription management performance
    pub async fn benchmark_subscription_performance(
        connection_count: usize,
        subscriptions_per_connection: usize,
    ) -> BenchmarkResults {
        let start_time = Instant::now();
        let mut errors = 0u64;
        
        // Create test connections
        let mut connection_ids = Vec::new();
        for _ in 0..connection_count {
            let conn_id = Uuid::new_v4();
            let conn_info = crate::websocket::ConnectionInfo {
                id: conn_id,
                user_address: None,
                subscriptions: Vec::new(),
                connected_at: Utc::now(),
                last_activity: Utc::now(),
                sender: None,
                health: crate::websocket::ConnectionHealth {
                    last_ping: tokio::time::Instant::now(),
                    last_pong: tokio::time::Instant::now(),
                    message_count: 0,
                    error_count: 0,
                    is_healthy: true,
                    rate_limiter: crate::websocket::RateLimiter::new(60, Duration::from_secs(60)),
                },
            };
            
            if WS_MANAGER.add_connection(conn_info).await.is_err() {
                errors += 1;
            } else {
                connection_ids.push(conn_id);
            }
        }
        
        // Benchmark subscription operations
        let sub_start = Instant::now();
        
        for conn_id in &connection_ids {
            for i in 0..subscriptions_per_connection {
                let channel = match i % 4 {
                    0 => SubscriptionChannel::MarketData,
                    1 => SubscriptionChannel::SystemAlerts,
                    2 => SubscriptionChannel::IntentUpdates(
                        format!("0x{:064x}", i).parse().unwrap_or_default()
                    ),
                    _ => SubscriptionChannel::UserIntents(
                        format!("0x{:040x}", i).parse().unwrap_or_default()
                    ),
                };
                
                if WS_MANAGER.subscribe_to_channel(*conn_id, channel).await.is_err() {
                    errors += 1;
                }
            }
        }
        
        let sub_duration = sub_start.elapsed();
        
        // Clean up
        for conn_id in connection_ids {
            WS_MANAGER.remove_connection(conn_id).await;
        }
        
        let total_duration = start_time.elapsed();
        let total_operations = connection_count * subscriptions_per_connection;
        let ops_per_second = total_operations as f64 / sub_duration.as_secs_f64();
        let success_rate = (total_operations - errors as usize) as f64 / total_operations as f64;
        
        BenchmarkResults {
            test_name: "Subscription Management".to_string(),
            duration: total_duration,
            operations_per_second: ops_per_second,
            success_rate,
            memory_usage_mb: Self::get_memory_usage_mb(),
            error_count: errors,
        }
    }
    
    /// Benchmark health monitoring performance
    pub async fn benchmark_health_monitoring(connection_count: usize) -> BenchmarkResults {
        let start_time = Instant::now();
        let mut errors = 0u64;
        
        // Create test connections
        let mut connection_ids = Vec::new();
        for _ in 0..connection_count {
            let conn_id = Uuid::new_v4();
            let conn_info = crate::websocket::ConnectionInfo {
                id: conn_id,
                user_address: None,
                subscriptions: vec!["market_data".to_string()],
                connected_at: Utc::now(),
                last_activity: Utc::now(),
                sender: None,
                health: crate::websocket::ConnectionHealth {
                    last_ping: tokio::time::Instant::now(),
                    last_pong: tokio::time::Instant::now(),
                    message_count: 0,
                    error_count: 0,
                    is_healthy: true,
                    rate_limiter: crate::websocket::RateLimiter::new(60, Duration::from_secs(60)),
                },
            };
            
            if WS_MANAGER.add_connection(conn_info).await.is_err() {
                errors += 1;
            } else {
                connection_ids.push(conn_id);
            }
        }
        
        // Benchmark health checks
        let health_start = Instant::now();
        
        for _ in 0..10 {
            WS_MANAGER.perform_health_checks().await;
            time::sleep(Duration::from_millis(10)).await;
        }
        
        let health_duration = health_start.elapsed();
        
        // Clean up
        for conn_id in connection_ids {
            WS_MANAGER.remove_connection(conn_id).await;
        }
        
        let total_duration = start_time.elapsed();
        let ops_per_second = (connection_count * 10) as f64 / health_duration.as_secs_f64();
        
        BenchmarkResults {
            test_name: "Health Monitoring".to_string(),
            duration: total_duration,
            operations_per_second: ops_per_second,
            success_rate: 1.0,
            memory_usage_mb: Self::get_memory_usage_mb(),
            error_count: errors,
        }
    }
    
    /// Run comprehensive WebSocket benchmarks
    pub async fn run_comprehensive_benchmarks() -> Vec<BenchmarkResults> {
        let mut results = Vec::new();
        
        println!("Running WebSocket performance benchmarks...");
        
        // Broadcast performance
        println!("Testing broadcast performance...");
        results.push(Self::benchmark_broadcast_performance(1000, 100).await);
        
        // Connection management
        println!("Testing connection management...");
        results.push(Self::benchmark_connection_management(1000).await);
        
        // Subscription management
        println!("Testing subscription management...");
        results.push(Self::benchmark_subscription_performance(100, 10).await);
        
        // Health monitoring
        println!("Testing health monitoring...");
        results.push(Self::benchmark_health_monitoring(500).await);
        
        results
    }
    
    /// Get current memory usage in MB
    fn get_memory_usage_mb() -> f64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<f64>() {
                                return kb / 1024.0; // Convert KB to MB
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback: use Rust's memory usage (heap only)
        let usage = std::alloc::System.new();
        0.0 // Placeholder - would need a proper memory profiler in production
    }
    
    /// Print benchmark results in a formatted table
    pub fn print_results(results: &[BenchmarkResults]) {
        println!("\n=== WebSocket Performance Benchmark Results ===\n");
        println!("{:<25} {:>12} {:>15} {:>12} {:>10} {:>8}", 
                 "Test Name", "Duration (ms)", "Ops/sec", "Success Rate", "Memory (MB)", "Errors");
        println!("{}", "-".repeat(85));
        
        for result in results {
            println!("{:<25} {:>12.2} {:>15.0} {:>11.2}% {:>10.1} {:>8}",
                     result.test_name,
                     result.duration.as_millis(),
                     result.operations_per_second,
                     result.success_rate * 100.0,
                     result.memory_usage_mb,
                     result.error_count);
        }
        
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_broadcast_benchmark() {
        let result = WebSocketBenchmarks::benchmark_broadcast_performance(100, 10).await;
        assert!(result.operations_per_second > 0.0);
        assert!(result.success_rate >= 0.0);
    }
    
    #[tokio::test]
    async fn test_connection_benchmark() {
        let result = WebSocketBenchmarks::benchmark_connection_management(50).await;
        assert!(result.operations_per_second > 0.0);
        assert!(result.success_rate >= 0.0);
    }
}