//! Rust Intents System - Cross-Chain Intent Execution Platform
//! 
//! This library provides a complete implementation of a cross-chain intent
//! execution system with solver networks, reputation management, and MEV protection.

pub mod deployment;

// Re-export key types for convenience
pub use deployment::{
    deploy_to_holesky, DeploymentConfig, DeploymentResult, DeployedContracts, HoleskyDeployer,
    HOLESKY_CHAIN_ID, HOLESKY_RPC_URL,
};

/// System version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// System name
pub const NAME: &str = "Rust Intents System";

/// Get system information
pub fn system_info() -> String {
    format!("{} v{}", NAME, VERSION)
}