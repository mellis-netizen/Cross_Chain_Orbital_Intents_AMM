//! Performance monitoring and metrics collection for the solver executor
//! 
//! This module provides comprehensive monitoring capabilities including:
//! - Execution performance tracking
//! - Gas usage analytics
//! - Success/failure rate monitoring
//! - MEV protection effectiveness
//! - Bridge operation metrics

use crate::executor::{ExecutionMetrics, ExecutionStep};
use ethers::types::{U256, H256};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Detailed performance monitoring system
#[derive(Debug)]
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<DetailedMetrics>>,
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
    chain_metrics: Arc<RwLock<HashMap<u64, ChainMetrics>>>,
    protocol_metrics: Arc<RwLock<HashMap<String, ProtocolMetrics>>>,
    started_at: Instant,
}

/// Comprehensive metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedMetrics {
    // Basic execution metrics
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub cancelled_executions: u64,
    pub timeout_executions: u64,

    // Performance metrics
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub fastest_execution: Duration,
    pub slowest_execution: Duration,

    // Gas and cost metrics
    pub total_gas_used: U256,
    pub average_gas_per_execution: U256,
    pub total_bridge_fees: U256,
    pub average_bridge_fee: U256,

    // MEV protection metrics
    pub mev_protection_triggers: u64,
    pub mev_attacks_prevented: u64,
    pub average_protection_delay: Duration,

    // Error and recovery metrics
    pub rollback_operations: u64,
    pub retry_operations: u64,
    pub asset_lock_failures: u64,
    pub bridge_failures: u64,

    // Profitability metrics
    pub total_profit: U256,
    pub average_profit_bps: u64,
    pub unprofitable_intents: u64,

    // Uptime and availability
    pub uptime_percentage: f64,
    pub last_update: SystemTime,
}

/// Individual execution record for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub intent_id: H256,
    pub started_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub execution_time: Option<Duration>,
    pub final_step: ExecutionStep,
    pub gas_used: U256,
    pub bridge_fee: U256,
    pub profit: U256,
    pub source_chain: u64,
    pub dest_chain: u64,
    pub protocol_used: Option<String>,
    pub mev_protection_applied: bool,
    pub retry_count: u32,
    pub error_message: Option<String>,
}

/// Per-chain performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetrics {
    pub chain_id: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub average_execution_time: Duration,
    pub total_gas_used: U256,
    pub average_gas_price: U256,
    pub bridge_volume: U256,
    pub last_execution: Option<SystemTime>,
}

/// Per-protocol performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMetrics {
    pub protocol_name: String,
    pub total_volume: U256,
    pub total_executions: u64,
    pub success_rate: f64,
    pub average_slippage: f64,
    pub average_execution_time: Duration,
    pub total_fees: U256,
}

/// Real-time performance dashboard data
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceDashboard {
    pub current_metrics: DetailedMetrics,
    pub hourly_stats: Vec<HourlyStats>,
    pub chain_breakdown: Vec<ChainMetrics>,
    pub protocol_breakdown: Vec<ProtocolMetrics>,
    pub recent_executions: Vec<ExecutionRecord>,
    pub alerts: Vec<PerformanceAlert>,
}

/// Hourly aggregated statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: SystemTime,
    pub executions: u64,
    pub success_rate: f64,
    pub average_time: Duration,
    pub total_volume: U256,
    pub total_profit: U256,
}

/// Performance alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlert {
    HighFailureRate { rate: f64, threshold: f64 },
    SlowExecutions { average_time: Duration, threshold: Duration },
    HighGasUsage { average_gas: U256, threshold: U256 },
    LowProfitability { profit_bps: u64, threshold: u64 },
    BridgeFailures { count: u64, timeframe: Duration },
    MemoryUsageHigh { usage_mb: u64, threshold: u64 },
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(DetailedMetrics::default())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            chain_metrics: Arc::new(RwLock::new(HashMap::new())),
            protocol_metrics: Arc::new(RwLock::new(HashMap::new())),
            started_at: Instant::now(),
        }
    }

    /// Record the start of an execution
    pub async fn record_execution_start(
        &self,
        intent_id: H256,
        source_chain: u64,
        dest_chain: u64,
    ) {
        let record = ExecutionRecord {
            intent_id,
            started_at: SystemTime::now(),
            completed_at: None,
            execution_time: None,
            final_step: ExecutionStep::ValidatingIntent,
            gas_used: U256::zero(),
            bridge_fee: U256::zero(),
            profit: U256::zero(),
            source_chain,
            dest_chain,
            protocol_used: None,
            mev_protection_applied: false,
            retry_count: 0,
            error_message: None,
        };

        let mut history = self.execution_history.write().await;
        history.push(record);

        // Keep only last 1000 records
        if history.len() > 1000 {
            history.remove(0);
        }

        debug!("Started tracking execution for intent {}", intent_id);
    }

    /// Record execution completion
    pub async fn record_execution_complete(
        &self,
        intent_id: H256,
        success: bool,
        final_step: ExecutionStep,
        gas_used: U256,
        bridge_fee: U256,
        profit: U256,
        protocol_used: Option<String>,
        retry_count: u32,
        error_message: Option<String>,
    ) {
        let completed_at = SystemTime::now();

        // Update execution record
        {
            let mut history = self.execution_history.write().await;
            if let Some(record) = history.iter_mut().rev().find(|r| r.intent_id == intent_id) {
                record.completed_at = Some(completed_at);
                record.execution_time = Some(completed_at.duration_since(record.started_at).unwrap_or_default());
                record.final_step = final_step;
                record.gas_used = gas_used;
                record.bridge_fee = bridge_fee;
                record.profit = profit;
                record.protocol_used = protocol_used.clone();
                record.retry_count = retry_count;
                record.error_message = error_message.clone();
            }
        }

        // Update aggregate metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_executions += 1;

            if success {
                metrics.successful_executions += 1;
            } else {
                metrics.failed_executions += 1;
            }

            // Update gas metrics
            metrics.total_gas_used = metrics.total_gas_used + gas_used;
            metrics.average_gas_per_execution = if metrics.total_executions > 0 {
                metrics.total_gas_used / U256::from(metrics.total_executions)
            } else {
                U256::zero()
            };

            // Update bridge fee metrics
            metrics.total_bridge_fees = metrics.total_bridge_fees + bridge_fee;
            metrics.average_bridge_fee = if metrics.total_executions > 0 {
                metrics.total_bridge_fees / U256::from(metrics.total_executions)
            } else {
                U256::zero()
            };

            // Update profit metrics
            metrics.total_profit = metrics.total_profit + profit;

            // Update retry metrics
            if retry_count > 0 {
                metrics.retry_operations += retry_count as u64;
            }

            metrics.last_update = SystemTime::now();
        }

        // Update protocol metrics
        if let Some(protocol) = protocol_used {
            self.update_protocol_metrics(protocol, success, gas_used, profit).await;
        }

        info!("Recorded execution completion for intent {} - Success: {}", intent_id, success);
    }

    /// Record MEV protection application
    pub async fn record_mev_protection(&self, delay: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.mev_protection_triggers += 1;

        // Update average protection delay
        let total_delay = metrics.average_protection_delay
            .mul_f64(metrics.mev_protection_triggers.saturating_sub(1) as f64)
            + delay;
        metrics.average_protection_delay = total_delay.div_f64(metrics.mev_protection_triggers as f64);

        debug!("Recorded MEV protection with {} second delay", delay.as_secs());
    }

    /// Record rollback operation
    pub async fn record_rollback(&self, intent_id: H256, reason: String) {
        let mut metrics = self.metrics.write().await;
        metrics.rollback_operations += 1;

        warn!("Recorded rollback for intent {}: {}", intent_id, reason);
    }

    /// Update chain-specific metrics
    async fn update_chain_metrics(&self, chain_id: u64, execution_time: Duration, gas_used: U256) {
        let mut chain_metrics = self.chain_metrics.write().await;
        let chain_metric = chain_metrics.entry(chain_id).or_insert_with(|| ChainMetrics {
            chain_id,
            total_executions: 0,
            successful_executions: 0,
            average_execution_time: Duration::from_secs(0),
            total_gas_used: U256::zero(),
            average_gas_price: U256::zero(),
            bridge_volume: U256::zero(),
            last_execution: None,
        });

        chain_metric.total_executions += 1;
        chain_metric.total_gas_used = chain_metric.total_gas_used + gas_used;
        chain_metric.last_execution = Some(SystemTime::now());

        // Update average execution time
        let total_time = chain_metric.average_execution_time
            .mul_f64(chain_metric.total_executions.saturating_sub(1) as f64)
            + execution_time;
        chain_metric.average_execution_time = total_time.div_f64(chain_metric.total_executions as f64);
    }

    /// Update protocol-specific metrics
    async fn update_protocol_metrics(
        &self,
        protocol: String,
        success: bool,
        gas_used: U256,
        profit: U256,
    ) {
        let mut protocol_metrics = self.protocol_metrics.write().await;
        let protocol_metric = protocol_metrics.entry(protocol.clone()).or_insert_with(|| ProtocolMetrics {
            protocol_name: protocol.clone(),
            total_volume: U256::zero(),
            total_executions: 0,
            success_rate: 0.0,
            average_slippage: 0.0,
            average_execution_time: Duration::from_secs(0),
            total_fees: U256::zero(),
        });

        protocol_metric.total_executions += 1;
        protocol_metric.total_volume = protocol_metric.total_volume + profit;

        // Update success rate
        let successful = if success { 1 } else { 0 };
        protocol_metric.success_rate = (protocol_metric.success_rate 
            * (protocol_metric.total_executions.saturating_sub(1) as f64) 
            + successful as f64) / protocol_metric.total_executions as f64;
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> DetailedMetrics {
        self.metrics.read().await.clone()
    }

    /// Get performance dashboard data
    pub async fn get_dashboard(&self) -> PerformanceDashboard {
        let current_metrics = self.get_metrics().await;
        let hourly_stats = self.generate_hourly_stats().await;
        let chain_breakdown = self.get_chain_metrics().await;
        let protocol_breakdown = self.get_protocol_metrics().await;
        let recent_executions = self.get_recent_executions(10).await;
        let alerts = self.generate_alerts(&current_metrics).await;

        PerformanceDashboard {
            current_metrics,
            hourly_stats,
            chain_breakdown,
            protocol_breakdown,
            recent_executions,
            alerts,
        }
    }

    /// Generate hourly statistics
    async fn generate_hourly_stats(&self) -> Vec<HourlyStats> {
        let history = self.execution_history.read().await;
        let mut hourly_map = HashMap::new();

        for record in history.iter() {
            if let Some(completed_at) = record.completed_at {
                let hour = completed_at.duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() / 3600 * 3600; // Round to hour

                let hour_time = UNIX_EPOCH + Duration::from_secs(hour);
                
                let stats = hourly_map.entry(hour_time).or_insert_with(|| HourlyStats {
                    hour: hour_time,
                    executions: 0,
                    success_rate: 0.0,
                    average_time: Duration::from_secs(0),
                    total_volume: U256::zero(),
                    total_profit: U256::zero(),
                });

                stats.executions += 1;
                stats.total_profit = stats.total_profit + record.profit;
                
                if let Some(exec_time) = record.execution_time {
                    let total_time = stats.average_time.mul_f64(stats.executions.saturating_sub(1) as f64) + exec_time;
                    stats.average_time = total_time.div_f64(stats.executions as f64);
                }
            }
        }

        let mut hourly_stats: Vec<_> = hourly_map.into_values().collect();
        hourly_stats.sort_by_key(|s| s.hour);
        hourly_stats.into_iter().rev().take(24).collect() // Last 24 hours
    }

    /// Get chain metrics
    async fn get_chain_metrics(&self) -> Vec<ChainMetrics> {
        let chain_metrics = self.chain_metrics.read().await;
        chain_metrics.values().cloned().collect()
    }

    /// Get protocol metrics
    async fn get_protocol_metrics(&self) -> Vec<ProtocolMetrics> {
        let protocol_metrics = self.protocol_metrics.read().await;
        protocol_metrics.values().cloned().collect()
    }

    /// Get recent execution records
    async fn get_recent_executions(&self, count: usize) -> Vec<ExecutionRecord> {
        let history = self.execution_history.read().await;
        history.iter().rev().take(count).cloned().collect()
    }

    /// Generate performance alerts
    async fn generate_alerts(&self, metrics: &DetailedMetrics) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();

        // High failure rate alert
        if metrics.total_executions > 10 {
            let failure_rate = metrics.failed_executions as f64 / metrics.total_executions as f64;
            if failure_rate > 0.2 { // 20% failure rate threshold
                alerts.push(PerformanceAlert::HighFailureRate {
                    rate: failure_rate,
                    threshold: 0.2,
                });
            }
        }

        // Slow execution alert
        if metrics.average_execution_time > Duration::from_secs(180) { // 3 minutes threshold
            alerts.push(PerformanceAlert::SlowExecutions {
                average_time: metrics.average_execution_time,
                threshold: Duration::from_secs(180),
            });
        }

        // High gas usage alert
        let gas_threshold = U256::from(500_000); // 500k gas threshold
        if metrics.average_gas_per_execution > gas_threshold {
            alerts.push(PerformanceAlert::HighGasUsage {
                average_gas: metrics.average_gas_per_execution,
                threshold: gas_threshold,
            });
        }

        alerts
    }

    /// Export metrics to JSON
    pub async fn export_metrics(&self) -> Result<String, serde_json::Error> {
        let dashboard = self.get_dashboard().await;
        serde_json::to_string_pretty(&dashboard)
    }

    /// Reset all metrics (for testing or maintenance)
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = DetailedMetrics::default();
        
        let mut history = self.execution_history.write().await;
        history.clear();
        
        let mut chain_metrics = self.chain_metrics.write().await;
        chain_metrics.clear();
        
        let mut protocol_metrics = self.protocol_metrics.write().await;
        protocol_metrics.clear();

        info!("Performance metrics reset");
    }
}

impl Default for DetailedMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            cancelled_executions: 0,
            timeout_executions: 0,
            total_execution_time: Duration::from_secs(0),
            average_execution_time: Duration::from_secs(0),
            fastest_execution: Duration::from_secs(u64::MAX),
            slowest_execution: Duration::from_secs(0),
            total_gas_used: U256::zero(),
            average_gas_per_execution: U256::zero(),
            total_bridge_fees: U256::zero(),
            average_bridge_fee: U256::zero(),
            mev_protection_triggers: 0,
            mev_attacks_prevented: 0,
            average_protection_delay: Duration::from_secs(0),
            rollback_operations: 0,
            retry_operations: 0,
            asset_lock_failures: 0,
            bridge_failures: 0,
            total_profit: U256::zero(),
            average_profit_bps: 0,
            unprofitable_intents: 0,
            uptime_percentage: 100.0,
            last_update: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::types::{Address, U256, H256};

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let metrics = monitor.get_metrics().await;
        
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.successful_executions, 0);
        assert_eq!(metrics.failed_executions, 0);
    }

    #[tokio::test]
    async fn test_execution_recording() {
        let monitor = PerformanceMonitor::new();
        let intent_id = H256::from_low_u64_be(1);

        // Record start
        monitor.record_execution_start(intent_id, 1, 137).await;

        // Record completion
        monitor.record_execution_complete(
            intent_id,
            true,
            ExecutionStep::Completed,
            U256::from(21000),
            U256::from(1000),
            U256::from(5000),
            Some("orbital_amm".to_string()),
            0,
            None,
        ).await;

        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.total_gas_used, U256::from(21000));
    }

    #[tokio::test]
    async fn test_mev_protection_recording() {
        let monitor = PerformanceMonitor::new();
        let delay = Duration::from_secs(5);

        monitor.record_mev_protection(delay).await;

        let metrics = monitor.get_metrics().await;
        assert_eq!(metrics.mev_protection_triggers, 1);
        assert_eq!(metrics.average_protection_delay, delay);
    }

    #[test]
    fn test_detailed_metrics_default() {
        let metrics = DetailedMetrics::default();
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.uptime_percentage, 100.0);
        assert_eq!(metrics.fastest_execution, Duration::from_secs(u64::MAX));
        assert_eq!(metrics.slowest_execution, Duration::from_secs(0));
    }

    #[test]
    fn test_performance_alert_types() {
        let alert = PerformanceAlert::HighFailureRate { rate: 0.3, threshold: 0.2 };
        match alert {
            PerformanceAlert::HighFailureRate { rate, threshold } => {
                assert_eq!(rate, 0.3);
                assert_eq!(threshold, 0.2);
            }
            _ => panic!("Wrong alert type"),
        }
    }
}