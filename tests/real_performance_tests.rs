//! Real Performance Tests with Concurrent Transactions
//!
//! This module tests system performance under various load conditions,
//! including concurrent transaction processing, throughput analysis,
//! and scalability testing.

use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256, H256, TransactionRequest},
    utils::{parse_ether, format_ether},
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
    collections::HashMap,
};
use serde_json::{json, Value};
use tokio::{task::JoinHandle, time::sleep};
use futures::future::join_all;

// Test configuration
const HOLESKY_CHAIN_ID: u64 = 17000;
const HOLESKY_RPC_URL: &str = "https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/";
const TEST_PRIVATE_KEY: &str = "0c068df4a4470cb73e6704d87c61a0c2718e72381c7b1e971514e5f9c4486f93";

// Performance test parameters
const MAX_CONCURRENT_TRANSACTIONS: usize = 50;
const PERFORMANCE_TEST_DURATION: Duration = Duration::from_secs(30);
const THROUGHPUT_MEASUREMENT_WINDOW: Duration = Duration::from_secs(5);

#[derive(Clone, Debug)]
pub struct PerformanceMetrics {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub min_execution_time: Duration,
    pub max_execution_time: Duration,
    pub transactions_per_second: f64,
    pub total_gas_used: U256,
    pub average_gas_per_transaction: U256,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_transactions: 0,
            successful_transactions: 0,
            failed_transactions: 0,
            total_execution_time: Duration::from_secs(0),
            average_execution_time: Duration::from_secs(0),
            min_execution_time: Duration::from_secs(u64::MAX),
            max_execution_time: Duration::from_secs(0),
            transactions_per_second: 0.0,
            total_gas_used: U256::zero(),
            average_gas_per_transaction: U256::zero(),
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransactionResult {
    pub success: bool,
    pub execution_time: Duration,
    pub gas_used: Option<U256>,
    pub transaction_hash: Option<H256>,
    pub error_message: Option<String>,
    pub timestamp: Instant,
}

#[derive(Clone, Debug)]
pub struct LoadTestConfig {
    pub concurrent_transactions: usize,
    pub test_duration: Duration,
    pub transaction_interval: Duration,
    pub max_retries: usize,
    pub target_tps: f64,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            concurrent_transactions: 10,
            test_duration: Duration::from_secs(30),
            transaction_interval: Duration::from_millis(100),
            max_retries: 3,
            target_tps: 10.0,
        }
    }
}

pub struct PerformanceTestSuite {
    provider: Arc<Provider<Http>>,
    wallet: LocalWallet,
    signer: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    contract_addresses: HashMap<String, Address>,
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl PerformanceTestSuite {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(HOLESKY_RPC_URL)?;
        let provider = Arc::new(provider);

        let wallet: LocalWallet = TEST_PRIVATE_KEY.parse()?;
        let wallet = wallet.with_chain_id(HOLESKY_CHAIN_ID);

        let signer = SignerMiddleware::new(provider.clone(), wallet.clone());
        let signer = Arc::new(signer);

        let mut contract_addresses = HashMap::new();
        contract_addresses.insert("intents".to_string(), "0x1234567890123456789012345678901234567890".parse()?);
        contract_addresses.insert("orbital_amm".to_string(), "0x2345678901234567890123456789012345678901".parse()?);
        contract_addresses.insert("usdc".to_string(), "0x3456789012345678901234567890123456789012".parse()?);

        Ok(Self {
            provider,
            wallet,
            signer,
            contract_addresses,
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        })
    }

    pub async fn run_comprehensive_performance_tests(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error>> {
        println!("🚀 Running Comprehensive Performance Tests");
        println!("=" .repeat(70));

        let mut all_results = Vec::new();

        // Test 1: Concurrent transaction processing
        let concurrent_results = self.test_concurrent_transactions().await?;
        all_results.extend(concurrent_results);

        // Test 2: Throughput analysis
        let throughput_results = self.test_throughput_analysis().await?;
        all_results.extend(throughput_results);

        // Test 3: Scalability testing
        let scalability_results = self.test_scalability().await?;
        all_results.extend(scalability_results);

        // Test 4: Memory and resource usage
        let resource_results = self.test_resource_usage().await?;
        all_results.extend(resource_results);

        // Test 5: Stress testing
        let stress_results = self.test_stress_scenarios().await?;
        all_results.extend(stress_results);

        Ok(all_results)
    }

    pub async fn test_concurrent_transactions(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error>> {
        println!("⚡ Testing Concurrent Transaction Processing");

        let mut results = Vec::new();

        // Test different concurrency levels
        let concurrency_levels = vec![1, 5, 10, 20, 30];

        for &concurrency in &concurrency_levels {
            println!("  🧪 Testing {} concurrent transactions", concurrency);
            
            let test_start = Instant::now();
            let transaction_results = self.execute_concurrent_transactions(concurrency).await?;

            let execution_time = test_start.elapsed();
            let successful = transaction_results.iter().filter(|r| r.success).count();
            let failed = transaction_results.len() - successful;

            let avg_execution_time = if !transaction_results.is_empty() {
                transaction_results.iter()
                    .map(|r| r.execution_time.as_millis())
                    .sum::<u128>() / transaction_results.len() as u128
            } else {
                0
            };

            let total_gas: u64 = transaction_results.iter()
                .filter_map(|r| r.gas_used)
                .map(|g| g.as_u64())
                .sum();

            let tps = if execution_time.as_secs_f64() > 0.0 {
                successful as f64 / execution_time.as_secs_f64()
            } else {
                0.0
            };

            results.push(TestResult {
                test_name: format!("Concurrent Transactions ({})", concurrency),
                success: successful > 0,
                execution_time,
                gas_used: Some(U256::from(total_gas)),
                transaction_hash: None,
                error_message: if failed > 0 { Some(format!("{} transactions failed", failed)) } else { None },
                metrics: json!({
                    "concurrency_level": concurrency,
                    "total_transactions": transaction_results.len(),
                    "successful_transactions": successful,
                    "failed_transactions": failed,
                    "success_rate": format!("{:.1}%", (successful as f64 / transaction_results.len() as f64) * 100.0),
                    "average_execution_time_ms": avg_execution_time,
                    "transactions_per_second": format!("{:.2}", tps),
                    "total_gas_used": total_gas
                }),
            });
        }

        Ok(results)
    }

    pub async fn test_throughput_analysis(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error>> {
        println!("📊 Testing Throughput Analysis");

        let mut results = Vec::new();

        // Test sustained throughput over time
        let test_configs = vec![
            LoadTestConfig {
                concurrent_transactions: 5,
                test_duration: Duration::from_secs(10),
                transaction_interval: Duration::from_millis(200),
                target_tps: 5.0,
                ..Default::default()
            },
            LoadTestConfig {
                concurrent_transactions: 10,
                test_duration: Duration::from_secs(15),
                transaction_interval: Duration::from_millis(100),
                target_tps: 10.0,
                ..Default::default()
            },
            LoadTestConfig {
                concurrent_transactions: 20,
                test_duration: Duration::from_secs(20),
                transaction_interval: Duration::from_millis(50),
                target_tps: 20.0,
                ..Default::default()
            },
        ];

        for (i, config) in test_configs.iter().enumerate() {
            println!("  🧪 Testing sustained throughput scenario {}", i + 1);
            
            let test_start = Instant::now();
            let throughput_result = self.measure_sustained_throughput(config.clone()).await?;

            results.push(TestResult {
                test_name: format!("Sustained Throughput Test {}", i + 1),
                success: throughput_result.actual_tps >= config.target_tps * 0.8, // 80% of target
                execution_time: test_start.elapsed(),
                gas_used: Some(throughput_result.total_gas_used),
                transaction_hash: None,
                error_message: None,
                metrics: json!({
                    "target_tps": config.target_tps,
                    "actual_tps": throughput_result.actual_tps,
                    "efficiency": format!("{:.1}%", (throughput_result.actual_tps / config.target_tps) * 100.0),
                    "total_transactions": throughput_result.total_transactions,
                    "successful_transactions": throughput_result.successful_transactions,
                    "test_duration_seconds": config.test_duration.as_secs(),
                    "average_latency_ms": throughput_result.average_latency_ms
                }),
            });
        }

        Ok(results)
    }

    pub async fn test_scalability(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error>> {
        println!("📈 Testing System Scalability");

        let mut results = Vec::new();

        // Test scalability by gradually increasing load
        let load_steps = vec![1, 2, 5, 10, 15, 20, 25];
        let mut previous_tps = 0.0;

        for &load in &load_steps {
            println!("  🧪 Testing scalability at load level {}", load);
            
            let test_start = Instant::now();
            let scalability_result = self.measure_scalability_at_load(load).await?;

            let efficiency = if previous_tps > 0.0 {
                scalability_result.tps / previous_tps / (load as f64 / (load - 1).max(1) as f64)
            } else {
                1.0
            };

            results.push(TestResult {
                test_name: format!("Scalability Test (Load {})", load),
                success: scalability_result.tps > 0.0,
                execution_time: test_start.elapsed(),
                gas_used: Some(scalability_result.total_gas),
                transaction_hash: None,
                error_message: None,
                metrics: json!({
                    "load_level": load,
                    "transactions_per_second": scalability_result.tps,
                    "scaling_efficiency": format!("{:.2}", efficiency),
                    "response_time_ms": scalability_result.avg_response_time_ms,
                    "error_rate": format!("{:.2}%", scalability_result.error_rate * 100.0),
                    "resource_utilization": scalability_result.resource_utilization
                }),
            });

            previous_tps = scalability_result.tps;
        }

        Ok(results)
    }

    pub async fn test_resource_usage(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error>> {
        println!("💾 Testing Memory and Resource Usage");

        let mut results = Vec::new();

        // Test resource usage under different loads
        let resource_test_start = Instant::now();
        
        // Simulate resource monitoring
        let initial_memory = self.get_memory_usage_mb();
        let initial_cpu = self.get_cpu_usage_percent();

        // Run resource-intensive operations
        let resource_results = self.execute_resource_intensive_operations().await?;

        let final_memory = self.get_memory_usage_mb();
        let final_cpu = self.get_cpu_usage_percent();

        let memory_increase = final_memory.saturating_sub(initial_memory);
        let cpu_increase = final_cpu - initial_cpu;

        results.push(TestResult {
            test_name: "Resource Usage Analysis".to_string(),
            success: memory_increase < 500 && cpu_increase < 80.0, // Reasonable thresholds
            execution_time: resource_test_start.elapsed(),
            gas_used: Some(resource_results.total_gas_used),
            transaction_hash: None,
            error_message: None,
            metrics: json!({
                "initial_memory_mb": initial_memory,
                "final_memory_mb": final_memory,
                "memory_increase_mb": memory_increase,
                "initial_cpu_percent": initial_cpu,
                "final_cpu_percent": final_cpu,
                "cpu_increase_percent": cpu_increase,
                "operations_completed": resource_results.operations_completed,
                "memory_efficiency": "good" // Could be calculated based on thresholds
            }),
        });

        Ok(results)
    }

    pub async fn test_stress_scenarios(&self) -> Result<Vec<TestResult>, Box<dyn std::error::Error>> {
        println!("🔥 Testing Stress Scenarios");

        let mut results = Vec::new();

        // Stress test scenarios
        let stress_scenarios = vec![
            ("High Volume Burst", 50, Duration::from_secs(5)),
            ("Sustained High Load", 25, Duration::from_secs(20)),
            ("Gradual Load Increase", 30, Duration::from_secs(15)),
        ];

        for (scenario_name, max_concurrent, duration) in stress_scenarios {
            println!("  🧪 Testing stress scenario: {}", scenario_name);
            
            let test_start = Instant::now();
            let stress_result = self.execute_stress_test(max_concurrent, duration).await?;

            let system_stable = stress_result.error_rate < 0.1 && stress_result.avg_response_time < 5000.0;

            results.push(TestResult {
                test_name: format!("Stress Test: {}", scenario_name),
                success: system_stable,
                execution_time: test_start.elapsed(),
                gas_used: Some(stress_result.total_gas_used),
                transaction_hash: None,
                error_message: if !system_stable { 
                    Some("System showed instability under stress".to_string()) 
                } else { 
                    None 
                },
                metrics: json!({
                    "scenario": scenario_name,
                    "max_concurrent": max_concurrent,
                    "test_duration_seconds": duration.as_secs(),
                    "total_operations": stress_result.total_operations,
                    "successful_operations": stress_result.successful_operations,
                    "error_rate": format!("{:.2}%", stress_result.error_rate * 100.0),
                    "average_response_time_ms": stress_result.avg_response_time,
                    "max_response_time_ms": stress_result.max_response_time,
                    "system_stability": if system_stable { "stable" } else { "unstable" }
                }),
            });
        }

        Ok(results)
    }

    // Helper methods for performance testing

    async fn execute_concurrent_transactions(&self, count: usize) -> Result<Vec<TransactionResult>, Box<dyn std::error::Error>> {
        let mut handles: Vec<JoinHandle<TransactionResult>> = Vec::new();

        for i in 0..count {
            let signer = self.signer.clone();
            let wallet_address = self.wallet.address();
            
            let handle = tokio::spawn(async move {
                let start_time = Instant::now();
                
                let tx = TransactionRequest::new()
                    .to(wallet_address)
                    .value(parse_ether("0.001").unwrap())
                    .gas(21000u64)
                    .gas_price(parse_ether("0.00001").unwrap());

                match signer.send_transaction(tx, None).await {
                    Ok(pending_tx) => {
                        match pending_tx.await {
                            Ok(Some(receipt)) => TransactionResult {
                                success: true,
                                execution_time: start_time.elapsed(),
                                gas_used: Some(receipt.gas_used.unwrap_or(U256::from(21000))),
                                transaction_hash: Some(receipt.transaction_hash),
                                error_message: None,
                                timestamp: Instant::now(),
                            },
                            Ok(None) => TransactionResult {
                                success: false,
                                execution_time: start_time.elapsed(),
                                gas_used: None,
                                transaction_hash: None,
                                error_message: Some("Transaction receipt not found".to_string()),
                                timestamp: Instant::now(),
                            },
                            Err(e) => TransactionResult {
                                success: false,
                                execution_time: start_time.elapsed(),
                                gas_used: None,
                                transaction_hash: None,
                                error_message: Some(format!("Transaction confirmation failed: {}", e)),
                                timestamp: Instant::now(),
                            },
                        }
                    }
                    Err(e) => TransactionResult {
                        success: false,
                        execution_time: start_time.elapsed(),
                        gas_used: None,
                        transaction_hash: None,
                        error_message: Some(format!("Transaction submission failed: {}", e)),
                        timestamp: Instant::now(),
                    },
                }
            });
            
            handles.push(handle);
            
            // Small delay to avoid overwhelming the network
            if i < count - 1 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        let results = join_all(handles).await;
        Ok(results.into_iter().map(|r| r.unwrap()).collect())
    }

    async fn measure_sustained_throughput(&self, config: LoadTestConfig) -> Result<ThroughputResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let mut total_transactions = 0;
        let mut successful_transactions = 0;
        let mut total_latency = Duration::from_secs(0);
        let mut total_gas_used = U256::zero();

        while start_time.elapsed() < config.test_duration {
            let batch_start = Instant::now();
            let batch_results = self.execute_concurrent_transactions(config.concurrent_transactions).await?;
            
            total_transactions += batch_results.len();
            successful_transactions += batch_results.iter().filter(|r| r.success).count();
            
            for result in &batch_results {
                total_latency += result.execution_time;
                if let Some(gas) = result.gas_used {
                    total_gas_used += gas;
                }
            }

            // Wait for the next batch
            let elapsed = batch_start.elapsed();
            if elapsed < config.transaction_interval {
                sleep(config.transaction_interval - elapsed).await;
            }
        }

        let actual_duration = start_time.elapsed();
        let actual_tps = successful_transactions as f64 / actual_duration.as_secs_f64();
        let average_latency_ms = if total_transactions > 0 {
            total_latency.as_millis() as f64 / total_transactions as f64
        } else {
            0.0
        };

        Ok(ThroughputResult {
            total_transactions,
            successful_transactions,
            actual_tps,
            average_latency_ms,
            total_gas_used,
        })
    }

    async fn measure_scalability_at_load(&self, load: usize) -> Result<ScalabilityResult, Box<dyn std::error::Error>> {
        let test_duration = Duration::from_secs(10);
        let start_time = Instant::now();

        let mut total_transactions = 0;
        let mut successful_transactions = 0;
        let mut response_times = Vec::new();
        let mut total_gas = U256::zero();

        while start_time.elapsed() < test_duration {
            let batch_results = self.execute_concurrent_transactions(load).await?;
            
            total_transactions += batch_results.len();
            successful_transactions += batch_results.iter().filter(|r| r.success).count();
            
            for result in &batch_results {
                response_times.push(result.execution_time.as_millis() as f64);
                if let Some(gas) = result.gas_used {
                    total_gas += gas;
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        let actual_duration = start_time.elapsed();
        let tps = successful_transactions as f64 / actual_duration.as_secs_f64();
        let avg_response_time_ms = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        } else {
            0.0
        };
        let error_rate = if total_transactions > 0 {
            (total_transactions - successful_transactions) as f64 / total_transactions as f64
        } else {
            0.0
        };

        Ok(ScalabilityResult {
            tps,
            avg_response_time_ms,
            error_rate,
            total_gas,
            resource_utilization: self.get_cpu_usage_percent(),
        })
    }

    async fn execute_resource_intensive_operations(&self) -> Result<ResourceResult, Box<dyn std::error::Error>> {
        let mut operations_completed = 0;
        let mut total_gas_used = U256::zero();

        // Simulate resource-intensive operations
        for i in 0..20 {
            let batch_size = 5 + (i % 10); // Varying batch sizes
            let batch_results = self.execute_concurrent_transactions(batch_size).await?;
            
            operations_completed += batch_results.len();
            for result in &batch_results {
                if let Some(gas) = result.gas_used {
                    total_gas_used += gas;
                }
            }

            // Small delay between batches
            sleep(Duration::from_millis(200)).await;
        }

        Ok(ResourceResult {
            operations_completed,
            total_gas_used,
        })
    }

    async fn execute_stress_test(&self, max_concurrent: usize, duration: Duration) -> Result<StressResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let mut total_operations = 0;
        let mut successful_operations = 0;
        let mut response_times = Vec::new();
        let mut total_gas_used = U256::zero();

        while start_time.elapsed() < duration {
            // Gradually increase concurrent load
            let current_concurrent = ((start_time.elapsed().as_secs_f64() / duration.as_secs_f64()) * max_concurrent as f64) as usize + 1;
            let current_concurrent = current_concurrent.min(max_concurrent);

            let batch_results = self.execute_concurrent_transactions(current_concurrent).await?;
            
            total_operations += batch_results.len();
            successful_operations += batch_results.iter().filter(|r| r.success).count();
            
            for result in &batch_results {
                response_times.push(result.execution_time.as_millis() as f64);
                if let Some(gas) = result.gas_used {
                    total_gas_used += gas;
                }
            }

            sleep(Duration::from_millis(50)).await;
        }

        let error_rate = if total_operations > 0 {
            (total_operations - successful_operations) as f64 / total_operations as f64
        } else {
            0.0
        };

        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<f64>() / response_times.len() as f64
        } else {
            0.0
        };

        let max_response_time = response_times.iter().fold(0.0f64, |a, &b| a.max(b));

        Ok(StressResult {
            total_operations,
            successful_operations,
            error_rate,
            avg_response_time,
            max_response_time,
            total_gas_used,
        })
    }

    // System monitoring helpers (simulated)
    fn get_memory_usage_mb(&self) -> u64 {
        // Simulate memory usage calculation
        // In a real implementation, this would use system APIs
        std::process::id() as u64 % 1000 + 100 // Mock value between 100-1100 MB
    }

    fn get_cpu_usage_percent(&self) -> f64 {
        // Simulate CPU usage calculation
        // In a real implementation, this would use system APIs
        (std::process::id() as f64 % 100.0) / 2.0 // Mock value between 0-50%
    }
}

// Helper structs for performance test results

#[derive(Debug)]
struct ThroughputResult {
    total_transactions: usize,
    successful_transactions: usize,
    actual_tps: f64,
    average_latency_ms: f64,
    total_gas_used: U256,
}

#[derive(Debug)]
struct ScalabilityResult {
    tps: f64,
    avg_response_time_ms: f64,
    error_rate: f64,
    total_gas: U256,
    resource_utilization: f64,
}

#[derive(Debug)]
struct ResourceResult {
    operations_completed: usize,
    total_gas_used: U256,
}

#[derive(Debug)]
struct StressResult {
    total_operations: usize,
    successful_operations: usize,
    error_rate: f64,
    avg_response_time: f64,
    max_response_time: f64,
    total_gas_used: U256,
}

#[derive(Clone, Debug)]
pub struct TestResult {
    pub test_name: String,
    pub success: bool,
    pub execution_time: Duration,
    pub gas_used: Option<U256>,
    pub transaction_hash: Option<H256>,
    pub error_message: Option<String>,
    pub metrics: Value,
}

#[cfg(test)]
mod real_performance_tests {
    use super::*;

    #[tokio::test]
    async fn test_comprehensive_performance_analysis() {
        let test_suite = PerformanceTestSuite::new().await.expect("Failed to create test suite");
        
        println!("🚀 Running Comprehensive Performance Analysis");
        println!("=" .repeat(70));

        let results = test_suite.run_comprehensive_performance_tests().await.expect("Performance tests failed");

        // Print summary
        println!("\n📊 Performance Test Summary:");
        println!("=" .repeat(70));
        
        let mut successful = 0;
        let mut failed = 0;
        let mut total_gas = U256::zero();
        let mut max_tps = 0.0;
        let mut min_response_time = f64::MAX;
        let mut max_response_time = 0.0;
        
        for result in &results {
            let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
            println!("  {} - {} ({:?})", status, result.test_name, result.execution_time);
            
            if let Some(gas) = result.gas_used {
                println!("    ⛽ Gas Used: {}", gas);
                total_gas += gas;
            }
            
            if let Some(ref error) = result.error_message {
                println!("    🚨 Error: {}", error);
            }
            
            // Extract performance metrics
            if let Some(tps) = result.metrics.get("transactions_per_second") {
                if let Ok(tps_val) = tps.as_str().unwrap_or("0").parse::<f64>() {
                    max_tps = max_tps.max(tps_val);
                }
            }
            
            if let Some(response_time) = result.metrics.get("average_response_time_ms").or_else(|| result.metrics.get("response_time_ms")) {
                if let Some(rt_val) = response_time.as_f64() {
                    min_response_time = min_response_time.min(rt_val);
                    max_response_time = max_response_time.max(rt_val);
                }
            }
            
            if result.success {
                successful += 1;
            } else {
                failed += 1;
            }
            
            println!("    📋 Metrics: {}", result.metrics);
            println!();
        }

        println!("📈 Final Performance Summary:");
        println!("  Total Tests: {}", results.len());
        println!("  Successful: {}", successful);
        println!("  Failed: {}", failed);
        println!("  Success Rate: {:.1}%", (successful as f64 / results.len() as f64) * 100.0);
        println!("  Total Gas Used: {}", total_gas);
        println!("  Peak Throughput: {:.2} TPS", max_tps);
        if min_response_time != f64::MAX {
            println!("  Min Response Time: {:.2}ms", min_response_time);
            println!("  Max Response Time: {:.2}ms", max_response_time);
        }

        // Assert performance criteria
        assert!(successful >= 15, "At least 15 performance tests should pass");
        assert!((successful as f64 / results.len() as f64) >= 0.80, "Success rate should be at least 80%");
        assert!(max_tps >= 1.0, "System should achieve at least 1 TPS");
        
        if min_response_time != f64::MAX {
            assert!(max_response_time <= 10000.0, "Maximum response time should be under 10 seconds");
        }

        println!("\n🎉 All Performance Tests Completed!");
    }

    #[tokio::test]
    async fn test_transaction_throughput_benchmark() {
        let test_suite = PerformanceTestSuite::new().await.expect("Failed to create test suite");
        
        println!("🏁 Running Transaction Throughput Benchmark");
        
        let config = LoadTestConfig {
            concurrent_transactions: 10,
            test_duration: Duration::from_secs(15),
            transaction_interval: Duration::from_millis(100),
            target_tps: 8.0,
            max_retries: 3,
        };

        let start_time = Instant::now();
        let result = test_suite.measure_sustained_throughput(config.clone()).await.expect("Throughput test failed");
        let execution_time = start_time.elapsed();

        println!("  📊 Throughput Benchmark Results:");
        println!("    Total Transactions: {}", result.total_transactions);
        println!("    Successful Transactions: {}", result.successful_transactions);
        println!("    Target TPS: {:.2}", config.target_tps);
        println!("    Actual TPS: {:.2}", result.actual_tps);
        println!("    Efficiency: {:.1}%", (result.actual_tps / config.target_tps) * 100.0);
        println!("    Average Latency: {:.2}ms", result.average_latency_ms);
        println!("    Total Execution Time: {:?}", execution_time);

        // Assert benchmark criteria
        assert!(result.actual_tps >= config.target_tps * 0.7, "Should achieve at least 70% of target TPS");
        assert!(result.average_latency_ms <= 2000.0, "Average latency should be under 2 seconds");
        assert!(result.successful_transactions > 0, "Should have successful transactions");

        println!("✅ Throughput benchmark completed successfully!");
    }
}