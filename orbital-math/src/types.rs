//! Error types for orbital math operations

use alloc::string::String;
use thiserror::Error;

/// Result type alias using [`OrbitalError`]
pub type Result<T> = core::result::Result<T, OrbitalError>;

/// Errors that can occur during orbital math operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum OrbitalError {
    /// Invalid number of tokens (must be between MIN_TOKENS and MAX_TOKENS)
    #[error("Invalid token count: {0}. Must be between 2 and 1000")]
    InvalidTokenCount(usize),

    /// Reserves don't satisfy the sphere constraint
    #[error("Sphere constraint violated: sum of squares {actual} != expected {expected}")]
    SphereConstraintViolation { actual: String, expected: String },

    /// Superellipse constraint violated
    #[error("Superellipse constraint violated with u={u}")]
    SuperellipseConstraintViolation { u: String },

    /// Zero reserve detected where positive value required
    #[error("Zero reserve for token {token_index}")]
    ZeroReserve { token_index: usize },

    /// Negative reserve (should never happen with U256 but checking overflow)
    #[error("Negative reserve detected for token {token_index}")]
    NegativeReserve { token_index: usize },

    /// Amount would cause reserves to violate invariant
    #[error("Trade amount {amount} would violate pool invariant")]
    InvariantViolation { amount: String },

    /// Insufficient liquidity for requested trade
    #[error("Insufficient liquidity: needed {needed}, available {available}")]
    InsufficientLiquidity { needed: String, available: String },

    /// Tick boundary crossed during calculation
    #[error("Tick boundary crossed unexpectedly at tick {tick_id}")]
    UnexpectedTickCrossing { tick_id: String },

    /// Invalid tick configuration
    #[error("Invalid tick: {reason}")]
    InvalidTick { reason: String },

    /// Tick overlap detected where it shouldn't exist
    #[error("Tick {tick_a} overlaps with tick {tick_b}")]
    TickOverlap { tick_a: String, tick_b: String },

    /// Mathematical overflow in calculation
    #[error("Overflow in calculation: {operation}")]
    Overflow { operation: String },

    /// Mathematical underflow in calculation
    #[error("Underflow in calculation: {operation}")]
    Underflow { operation: String },

    /// Division by zero
    #[error("Division by zero in operation: {operation}")]
    DivisionByZero { operation: String },

    /// Square root of negative number
    #[error("Square root of negative: {value}")]
    NegativeSquareRoot { value: String },

    /// Invalid parameter value
    #[error("Invalid parameter {param}: {reason}")]
    InvalidParameter { param: String, reason: String },

    /// Numerical precision loss too high
    #[error("Precision loss exceeds tolerance: {loss}%")]
    PrecisionLoss { loss: String },

    /// Token index out of bounds
    #[error("Token index {index} out of bounds (pool has {token_count} tokens)")]
    TokenIndexOutOfBounds { index: usize, token_count: usize },

    /// Price impact too high
    #[error("Price impact {impact}% exceeds maximum allowed {max}%")]
    ExcessivePriceImpact { impact: String, max: String },

    /// Slippage tolerance exceeded
    #[error("Slippage {actual}% exceeds tolerance {tolerance}%")]
    SlippageExceeded { actual: String, tolerance: String },

    /// Cannot solve equation (e.g., quartic equation has no real positive roots)
    #[error("Cannot solve {equation}: no valid solution found")]
    NoSolution { equation: String },

    /// Generic computation error
    #[error("Computation error: {details}")]
    ComputationError { details: String },
}

impl OrbitalError {
    /// Create an overflow error for a specific operation
    pub fn overflow(operation: impl Into<String>) -> Self {
        Self::Overflow {
            operation: operation.into(),
        }
    }

    /// Create an underflow error for a specific operation
    pub fn underflow(operation: impl Into<String>) -> Self {
        Self::Underflow {
            operation: operation.into(),
        }
    }

    /// Create a division by zero error for a specific operation
    pub fn division_by_zero(operation: impl Into<String>) -> Self {
        Self::DivisionByZero {
            operation: operation.into(),
        }
    }

    /// Create an invalid parameter error
    pub fn invalid_param(param: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidParameter {
            param: param.into(),
            reason: reason.into(),
        }
    }

    /// Create a computation error with details
    pub fn computation(details: impl Into<String>) -> Self {
        Self::ComputationError {
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = OrbitalError::overflow("test operation");
        assert!(matches!(err, OrbitalError::Overflow { .. }));

        let err = OrbitalError::invalid_param("test_param", "invalid value");
        assert!(matches!(err, OrbitalError::InvalidParameter { .. }));
    }

    #[test]
    fn test_error_display() {
        let err = OrbitalError::InvalidTokenCount(1);
        let display = format!("{}", err);
        assert!(display.contains("Invalid token count"));

        let err = OrbitalError::InsufficientLiquidity {
            needed: "1000".to_string(),
            available: "500".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Insufficient liquidity"));
    }
}