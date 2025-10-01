//! # Orbital Math Library
//!
//! Mathematical primitives for N-dimensional Orbital AMM pools.
//! Implements the core algorithms from Paradigm's Orbital research paper.
//!
//! ## Key Concepts
//!
//! - **Spherical Invariant**: Σ(r_i²) = R² - all reserve states lie on an N-dimensional sphere
//! - **Superellipse Variant**: Σ(|r_i|^u) = K - flattened curve for stablecoin trading
//! - **Tick System**: Hyperplane boundaries creating spherical caps for concentrated liquidity
//! - **Toroidal Trading**: Combined interior (sphere) and boundary (circle) liquidity
//!
//! ## Modules
//!
//! - [`sphere`]: Spherical AMM constraints and calculations
//! - [`superellipse`]: Superellipse curve mathematics
//! - [`ticks`]: Tick geometry and capital efficiency
//! - [`trades`]: Trade execution with tick boundary crossing
//! - [`types`]: Core types and traits used throughout the library

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::U256;

pub mod error;
pub mod sphere;
pub mod superellipse;
pub mod ticks;
pub mod trades;
pub mod types;
pub mod utils;
pub mod concentrated_liquidity;
pub mod ten_token_demo;

// Re-export commonly used types
pub use error::{OrbitalError, Result};
pub use types::*;

/// Precision for fixed-point arithmetic (18 decimals like ETH)
pub const PRECISION: u32 = 18;
pub const PRECISION_MULTIPLIER: u128 = 1_000_000_000_000_000_000;

/// Basis points precision (10000 = 100%)
pub const BP_PRECISION: u32 = 10000;

/// Maximum number of tokens supported in a pool
pub const MAX_TOKENS: usize = 1000;

/// Minimum number of tokens in a pool
pub const MIN_TOKENS: usize = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_constants() {
        assert_eq!(PRECISION, 18);
        assert_eq!(PRECISION_MULTIPLIER, 10u128.pow(PRECISION));
        assert_eq!(BP_PRECISION, 10000);
    }

    #[test]
    fn test_token_limits() {
        assert!(MIN_TOKENS >= 2);
        assert!(MAX_TOKENS <= 1000);
        assert!(MAX_TOKENS > MIN_TOKENS);
    }
}