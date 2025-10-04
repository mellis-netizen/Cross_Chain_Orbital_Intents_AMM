use intents_api::websocket_benchmarks::WebSocketBenchmarks;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🚀 WebSocket Performance Testing Suite");
    println!("======================================");
    
    // Run comprehensive benchmarks
    let results = WebSocketBenchmarks::run_comprehensive_benchmarks().await;
    
    // Print results
    WebSocketBenchmarks::print_results(&results);
    
    // Performance summary
    println!("\n📊 Performance Summary:");
    println!("=======================");
    
    for result in &results {
        match result.test_name.as_str() {
            "Broadcast Performance" => {
                println!("✅ Broadcast: {:.0} messages/sec with {:.1}% success rate", 
                         result.operations_per_second, result.success_rate * 100.0);
            }
            "Connection Management" => {
                println!("✅ Connections: {:.0} operations/sec with {:.1}% success rate", 
                         result.operations_per_second, result.success_rate * 100.0);
            }
            "Subscription Management" => {
                println!("✅ Subscriptions: {:.0} operations/sec with {:.1}% success rate", 
                         result.operations_per_second, result.success_rate * 100.0);
            }
            "Health Monitoring" => {
                println!("✅ Health Checks: {:.0} checks/sec", result.operations_per_second);
            }
            _ => {}
        }
    }
    
    // Performance targets
    println!("\n🎯 Performance Targets:");
    println!("=======================");
    
    let broadcast_perf = results.iter().find(|r| r.test_name == "Broadcast Performance");
    let connection_perf = results.iter().find(|r| r.test_name == "Connection Management");
    let subscription_perf = results.iter().find(|r| r.test_name == "Subscription Management");
    
    if let Some(broadcast) = broadcast_perf {
        if broadcast.operations_per_second >= 1000.0 && broadcast.success_rate >= 0.95 {
            println!("✅ Broadcast performance meets production targets");
        } else {
            println!("⚠️  Broadcast performance needs optimization");
        }
    }
    
    if let Some(connection) = connection_perf {
        if connection.operations_per_second >= 500.0 && connection.success_rate >= 0.99 {
            println!("✅ Connection management meets production targets");
        } else {
            println!("⚠️  Connection management needs optimization");
        }
    }
    
    if let Some(subscription) = subscription_perf {
        if subscription.operations_per_second >= 200.0 && subscription.success_rate >= 0.95 {
            println!("✅ Subscription management meets production targets");
        } else {
            println!("⚠️  Subscription management needs optimization");
        }
    }
    
    println!("\n🏆 Test completed successfully!");
    Ok(())
}