//! Superellipse AMM variant
//!
//! Implements superellipse curves optimized for stablecoin trading:
//! Σ(|r_i|^u) = K
//!
//! When u > 2, the curve is "flatter" than a circle, concentrating liquidity
//! around the equal price point (1:1 ratio for stablecoins).
//!
//! This implementation uses integer approximations for fractional powers.
//! For production, consider using a fixed-point math library or moving
//! complex calculations off-chain with on-chain verification.

use alloc::vec::Vec;
use alloy_primitives::U256;
use crate::{
    error::{OrbitalError, Result},
    utils::{pow, nth_root_approx, abs},
    MAX_TOKENS, MIN_TOKENS, BP_PRECISION,
};

/// Verify superellipse constraint: Σ(|r_i|^u) = K
///
/// # Arguments
/// * `reserves` - Current reserve amounts
/// * `u_parameter` - The u exponent scaled by 10000 (e.g., 25000 = 2.5)
/// * `k_constant` - The superellipse constant K
/// * `tolerance_bp` - Allowable deviation in basis points
///
/// # Returns
/// * `Ok(())` if constraint satisfied within tolerance
///
/// # Note
/// For u = 2, this is equivalent to a sphere (circle in 2D).
/// For u > 2, the curve is flatter around equal reserves.
pub fn verify_superellipse_constraint(
    reserves: &[U256],
    u_parameter: u32,  // Scaled by 10000
    k_constant: U256,
    tolerance_bp: u32,
) -> Result<()> {
    if reserves.len() < MIN_TOKENS || reserves.len() > MAX_TOKENS {
        return Err(OrbitalError::InvalidTokenCount(reserves.len()));
    }

    if u_parameter < 20000 {
        return Err(OrbitalError::invalid_param(
            "u_parameter",
            "must be >= 2.0 (20000)",
        ));
    }

    // For u = 2.0, use sphere verification
    if u_parameter == 20000 {
        let sum_of_squares = reserves
            .iter()
            .try_fold(U256::ZERO, |acc, &r| {
                let r_squared = r.checked_mul(r)
                    .ok_or_else(|| OrbitalError::overflow("reserve squared"))?;
                acc.checked_add(r_squared)
                    .ok_or_else(|| OrbitalError::overflow("sum"))
            })?;
        
        let tolerance = (k_constant * U256::from(tolerance_bp)) / U256::from(BP_PRECISION);
        let lower = k_constant.saturating_sub(tolerance);
        let upper = k_constant.saturating_add(tolerance);
        
        if sum_of_squares >= lower && sum_of_squares <= upper {
            return Ok(());
        } else {
            return Err(OrbitalError::SuperellipseConstraintViolation {
                u: u_parameter.to_string(),
            });
        }
    }

    // For u > 2, approximate with integer powers
    // Σ(r_i^u) ≈ K
    let u_int = u_parameter / 10000;
    let u_frac = u_parameter % 10000;

    let mut sum = U256::ZERO;
    
    for &reserve in reserves {
        // Calculate r^u_int (integer part)
        let r_int_power = pow(reserve, u_int)?;
        
        // For fractional part, approximate r^(u_frac/10000)
        // Using r^f ≈ r^(1/2) when f = 0.5
        // This is simplified; production should use better approximation
        let reserve_power = if u_frac == 0 {
            r_int_power
        } else if u_frac == 5000 {
            // u_frac = 0.5, so multiply by sqrt(r)
            let sqrt_r = crate::types::sqrt_approx(reserve);
            r_int_power.checked_mul(sqrt_r)
                .ok_or_else(|| OrbitalError::overflow("power with fraction"))?
        } else {
            // For other fractions, use integer approximation
            // This loses precision but works for demonstration
            r_int_power
        };

        sum = sum.checked_add(reserve_power)
            .ok_or_else(|| OrbitalError::overflow("sum of powers"))?;
    }

    // Check if sum ≈ k_constant within tolerance
    let tolerance = (k_constant * U256::from(tolerance_bp)) / U256::from(BP_PRECISION);
    let lower = k_constant.saturating_sub(tolerance);
    let upper = k_constant.saturating_add(tolerance);

    if sum >= lower && sum <= upper {
        Ok(())
    } else {
        Err(OrbitalError::SuperellipseConstraintViolation {
            u: u_parameter.to_string(),
        })
    }
}

/// Calculate swap output for superellipse curve
///
/// # Arguments
/// * `reserves` - Current reserves
/// * `token_in` - Input token index
/// * `token_out` - Output token index
/// * `amount_in` - Input amount
/// * `u_parameter` - Superellipse u parameter (scaled by 10000)
/// * `k_constant` - Superellipse K constant
///
/// # Returns
/// * Output amount that maintains the superellipse constraint
///
/// # Algorithm
/// Starting from: Σ(r_i^u) = K
/// After swap: (r_in + Δ_in)^u + (r_out - Δ_out)^u + Σ(r_k^u for k≠in,out) = K
/// Solve for Δ_out
pub fn calculate_amount_out_superellipse(
    reserves: &[U256],
    token_in: usize,
    token_out: usize,
    amount_in: U256,
    u_parameter: u32,
    k_constant: U256,
) -> Result<U256> {
    // Validate inputs
    if token_in >= reserves.len() || token_out >= reserves.len() {
        return Err(OrbitalError::TokenIndexOutOfBounds {
            index: token_in.max(token_out),
            token_count: reserves.len(),
        });
    }

    if token_in == token_out {
        return Err(OrbitalError::invalid_param(
            "tokens",
            "input and output must be different",
        ));
    }

    if amount_in.is_zero() {
        return Ok(U256::ZERO);
    }

    // For u = 2.0, use sphere calculation
    if u_parameter == 20000 {
        return crate::sphere::calculate_amount_out_sphere(
            reserves,
            token_in,
            token_out,
            amount_in,
            k_constant,
        );
    }

    // For u > 2, approximate the calculation
    let u_int = u_parameter / 10000;
    
    // Calculate new reserve for token_in
    let new_reserve_in = reserves[token_in]
        .checked_add(amount_in)
        .ok_or_else(|| OrbitalError::overflow("reserve_in + amount_in"))?;

    // Calculate (r_in + Δ_in)^u
    let new_reserve_in_power = pow(new_reserve_in, u_int)?;

    // Calculate Σ(r_k^u for k ≠ out)
    let mut sum_other_powers = U256::ZERO;
    for (i, &r) in reserves.iter().enumerate() {
        if i == token_out {
            continue;
        }
        
        let r_power = if i == token_in {
            new_reserve_in_power
        } else {
            pow(r, u_int)?
        };
        
        sum_other_powers = sum_other_powers
            .checked_add(r_power)
            .ok_or_else(|| OrbitalError::overflow("sum of powers"))?;
    }

    // K - Σ(r_k^u for k ≠ out) = (r_out - Δ_out)^u
    let remaining = k_constant
        .checked_sub(sum_other_powers)
        .ok_or_else(|| OrbitalError::InsufficientLiquidity {
            needed: amount_in.to_string(),
            available: reserves[token_out].to_string(),
        })?;

    // new_r_out = (remaining)^(1/u)
    let new_reserve_out = nth_root_approx(remaining, u_int)?;

    // Δ_out = r_out - new_r_out
    let amount_out = reserves[token_out]
        .checked_sub(new_reserve_out)
        .ok_or_else(|| OrbitalError::InsufficientLiquidity {
            needed: amount_in.to_string(),
            available: reserves[token_out].to_string(),
        })?;

    // Ensure we don't return more than available
    if amount_out > reserves[token_out] {
        return Err(OrbitalError::InsufficientLiquidity {
            needed: amount_out.to_string(),
            available: reserves[token_out].to_string(),
        });
    }

    Ok(amount_out)
}

/// Calculate instantaneous price on superellipse curve
///
/// For superellipse: Σ(r_i^u) = K
/// The price is: ∂r_j/∂r_i = -(r_i^(u-1))/(r_j^(u-1))
pub fn calculate_price_superellipse(
    reserves: &[U256],
    token_in: usize,
    token_out: usize,
    u_parameter: u32,
) -> Result<U256> {
    if token_in >= reserves.len() || token_out >= reserves.len() {
        return Err(OrbitalError::TokenIndexOutOfBounds {
            index: token_in.max(token_out),
            token_count: reserves.len(),
        });
    }

    let reserve_in = reserves[token_in];
    let reserve_out = reserves[token_out];

    if reserve_out.is_zero() {
        return Err(OrbitalError::DivisionByZero {
            operation: "price calculation".into(),
        });
    }

    // For u = 2.0, use sphere pricing
    if u_parameter == 20000 {
        return crate::sphere::calculate_price_sphere(reserves, token_in, token_out);
    }

    let u_int = u_parameter / 10000;
    
    // Calculate r_in^(u-1) and r_out^(u-1)
    let u_minus_1 = u_int.saturating_sub(1);
    
    let r_in_power = if u_minus_1 == 0 {
        U256::from(1)
    } else {
        pow(reserve_in, u_minus_1)?
    };
    
    let r_out_power = if u_minus_1 == 0 {
        U256::from(1)
    } else {
        pow(reserve_out, u_minus_1)?
    };

    // Price = r_in^(u-1) / r_out^(u-1) * PRECISION
    let price = r_in_power
        .checked_mul(U256::from(crate::PRECISION_MULTIPLIER))
        .ok_or_else(|| OrbitalError::overflow("price calculation"))?
        .checked_div(r_out_power)
        .ok_or_else(|| OrbitalError::division_by_zero("price calculation"))?;

    Ok(price)
}

/// Determine optimal u parameter based on expected volatility
///
/// # Arguments
/// * `volatility_bp` - Expected volatility in basis points (e.g., 50 = 0.5%)
///
/// # Returns
/// * Recommended u parameter (scaled by 10000)
///
/// # Guidelines
/// * High volatility (>100bp): u = 2.0-2.2 (more sphere-like)
/// * Medium volatility (50-100bp): u = 2.2-2.5
/// * Low volatility (<50bp): u = 2.5-3.0 (flatter, more concentrated)
pub fn optimal_u_for_volatility(volatility_bp: u32) -> u32 {
    if volatility_bp > 100 {
        // High volatility: more sphere-like
        22000  // u = 2.2
    } else if volatility_bp > 50 {
        // Medium volatility: balanced
        25000  // u = 2.5
    } else {
        // Low volatility: more concentrated
        28000  // u = 2.8
    }
}

/// Calculate the effective concentration ratio vs sphere
///
/// Higher u values concentrate more liquidity around equal price point
///
/// # Returns
/// * Concentration multiplier (scaled by 10000, e.g., 15000 = 1.5x)
pub fn concentration_ratio(u_parameter: u32) -> u32 {
    // Empirical formula: concentration ≈ u - 1
    // For u=2.5, concentration ≈ 1.5x
    // For u=3.0, concentration ≈ 2.0x
    
    if u_parameter <= 20000 {
        return 10000; // No concentration (sphere)
    }

    let u = u_parameter - 20000; // Subtract 2.0
    let concentration = 10000 + u; // Add to base
    
    // Cap at 3x concentration
    if concentration > 30000 {
        30000
    } else {
        concentration
    }
}

/// Convert superellipse parameters to equivalent sphere
///
/// Approximates a superellipse pool as a sphere for compatibility
pub fn superellipse_to_sphere_approximation(
    reserves: &[U256],
    u_parameter: u32,
    k_constant: U256,
) -> Result<U256> {
    // For sphere: Σ(r²) = R²
    // Approximate R² from superellipse K
    
    if u_parameter == 20000 {
        return Ok(k_constant); // Already a sphere
    }

    let u_int = u_parameter / 10000;
    
    // Rough approximation: R² ≈ K^(2/u) * N
    let n = U256::from(reserves.len());
    
    // For demonstration, use simplified formula
    // Production should use more accurate conversion
    let r_squared_approx = k_constant
        .checked_mul(n)
        .ok_or_else(|| OrbitalError::overflow("sphere approximation"))?;
    
    Ok(r_squared_approx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_superellipse_u2() {
        // u=2.0 should behave like sphere
        let reserves = vec![U256::from(3), U256::from(4)];
        // 3² + 4² = 25
        let k = U256::from(25);
        
        assert!(verify_superellipse_constraint(&reserves, 20000, k, 0).is_ok());
    }

    #[test]
    fn test_verify_superellipse_u25() {
        // u=2.5: more concentrated
        let reserves = vec![U256::from(10), U256::from(10), U256::from(10)];
        // 10^2.5 + 10^2.5 + 10^2.5 ≈ 3 * 316 ≈ 948
        let k = U256::from(1000); // Approximate
        
        // Should be within tolerance
        assert!(verify_superellipse_constraint(&reserves, 25000, k, 100).is_ok());
    }

    #[test]
    fn test_calculate_amount_out_u2() {
        // u=2.0 should match sphere calculation
        let reserves = vec![U256::from(100), U256::from(100)];
        let k = U256::from(20000);
        
        let amount_out = calculate_amount_out_superellipse(
            &reserves,
            0,
            1,
            U256::from(10),
            20000,
            k,
        ).unwrap();
        
        // Should return some positive amount
        assert!(amount_out > U256::ZERO);
        assert!(amount_out < reserves[1]);
    }

    #[test]
    fn test_optimal_u_for_volatility() {
        // High volatility
        assert_eq!(optimal_u_for_volatility(150), 22000);
        
        // Medium volatility
        assert_eq!(optimal_u_for_volatility(75), 25000);
        
        // Low volatility
        assert_eq!(optimal_u_for_volatility(25), 28000);
    }

    #[test]
    fn test_concentration_ratio() {
        // u=2.0 (sphere)
        assert_eq!(concentration_ratio(20000), 10000);
        
        // u=2.5
        assert_eq!(concentration_ratio(25000), 15000);
        
        // u=3.0
        assert_eq!(concentration_ratio(30000), 20000);
    }

    #[test]
    fn test_calculate_price_u2() {
        let reserves = vec![U256::from(100), U256::from(200)];
        
        let price = calculate_price_superellipse(&reserves, 0, 1, 20000).unwrap();
        
        // For u=2, price = 100/200 = 0.5
        let expected = U256::from(crate::PRECISION_MULTIPLIER / 2);
        assert_eq!(price, expected);
    }

    #[test]
    fn test_invalid_u_parameter() {
        let reserves = vec![U256::from(100), U256::from(100)];
        
        // u < 2.0 should fail
        let result = verify_superellipse_constraint(&reserves, 19000, U256::from(1000), 0);
        assert!(result.is_err());
    }
}