//! Spherical AMM invariant calculations
//!
//! Implements the core N-dimensional spherical constraint: Σ(r_i²) = R²
//!
//! This module provides the fundamental building block for Orbital AMMs where
//! all valid reserve states lie on the surface of an N-dimensional sphere.

use alloc::vec::Vec;
use alloy_primitives::U256;
use crate::{
    error::{OrbitalError, Result},
    types::{ReservePoint, PoolState, sqrt_approx},
    MAX_TOKENS, MIN_TOKENS,
};

/// Verify that reserves satisfy the sphere constraint: Σ(r_i²) = R²
///
/// # Arguments
/// * `reserves` - Current reserve amounts for all tokens
/// * `radius_squared` - The constant R² defining the sphere
/// * `tolerance_bp` - Allowable deviation in basis points (e.g., 10 = 0.1%)
///
/// # Returns
/// * `Ok(())` if constraint is satisfied within tolerance
/// * `Err` if constraint is violated
pub fn verify_sphere_constraint(
    reserves: &[U256],
    radius_squared: U256,
    tolerance_bp: u32,
) -> Result<()> {
    if reserves.len() < MIN_TOKENS || reserves.len() > MAX_TOKENS {
        return Err(OrbitalError::InvalidTokenCount(reserves.len()));
    }

    // Calculate Σ(r_i²)
    let sum_of_squares = reserves
        .iter()
        .try_fold(U256::ZERO, |acc, &r| {
            let r_squared = r.checked_mul(r)
                .ok_or_else(|| OrbitalError::overflow("reserve squared"))?;
            acc.checked_add(r_squared)
                .ok_or_else(|| OrbitalError::overflow("sum of squares"))
        })?;

    // Check if sum_of_squares ≈ radius_squared within tolerance
    let tolerance = (radius_squared * U256::from(tolerance_bp)) / U256::from(10000);
    let lower_bound = radius_squared.saturating_sub(tolerance);
    let upper_bound = radius_squared.saturating_add(tolerance);

    if sum_of_squares >= lower_bound && sum_of_squares <= upper_bound {
        Ok(())
    } else {
        Err(OrbitalError::SphereConstraintViolation {
            actual: sum_of_squares.to_string(),
            expected: radius_squared.to_string(),
        })
    }
}

/// Calculate the output amount for a trade on a spherical AMM
///
/// # Arguments
/// * `reserves` - Current reserve amounts
/// * `token_in` - Index of input token
/// * `token_out` - Index of output token
/// * `amount_in` - Amount of input token
/// * `radius_squared` - Pool's R² constant
///
/// # Returns
/// * Amount of output token that will be received
///
/// # Algorithm
/// Starting from reserves r⃗ on sphere, adding Δ_in to r_i and removing Δ_out from r_j
/// must keep us on the sphere:
/// (r_i + Δ_in)² + (r_j - Δ_out)² + Σ(r_k² for k ≠ i,j) = R²
///
/// Solving for Δ_out:
/// Δ_out = r_j - sqrt(R² - Σ(r_k² for k ≠ j) - (r_i + Δ_in)²)
pub fn calculate_amount_out_sphere(
    reserves: &[U256],
    token_in: usize,
    token_out: usize,
    amount_in: U256,
    radius_squared: U256,
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
            "token indices",
            "input and output must be different",
        ));
    }

    if amount_in.is_zero() {
        return Ok(U256::ZERO);
    }

    // Calculate new reserve for token_in after adding amount_in
    let new_reserve_in = reserves[token_in]
        .checked_add(amount_in)
        .ok_or_else(|| OrbitalError::overflow("reserve_in + amount_in"))?;

    // Calculate (r_i + Δ_in)²
    let new_reserve_in_squared = new_reserve_in
        .checked_mul(new_reserve_in)
        .ok_or_else(|| OrbitalError::overflow("new_reserve_in squared"))?;

    // Calculate Σ(r_k² for k ≠ j)
    let sum_other_squares = reserves
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != token_out)
        .try_fold(U256::ZERO, |acc, (i, &r)| {
            let r_sq = if i == token_in {
                new_reserve_in_squared
            } else {
                r.checked_mul(r)
                    .ok_or_else(|| OrbitalError::overflow(&format!("reserve[{}] squared", i)))?
            };
            acc.checked_add(r_sq)
                .ok_or_else(|| OrbitalError::overflow("sum of squares"))
        })?;

    // R² - Σ(r_k² for k ≠ j)
    let under_sqrt = radius_squared
        .checked_sub(sum_other_squares)
        .ok_or_else(|| OrbitalError::underflow("R² - sum_other_squares"))?;

    // sqrt(R² - Σ(r_k² for k ≠ j))
    let new_reserve_out = sqrt_approx(under_sqrt);

    // Δ_out = r_j - new_reserve_out
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

/// Calculate the instantaneous price of token_out in terms of token_in
///
/// # Arguments
/// * `reserves` - Current reserve amounts
/// * `token_in` - Index of input token  
/// * `token_out` - Index of output token
///
/// # Returns
/// * Price as (token_out per token_in) scaled by PRECISION
///
/// # Algorithm
/// From the sphere constraint, the instantaneous price (marginal rate) is:
/// ∂r_j/∂r_i = -r_i/r_j
///
/// So the price of token_out in terms of token_in is r_i/r_j
pub fn calculate_price_sphere(
    reserves: &[U256],
    token_in: usize,
    token_out: usize,
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

    // Price = reserve_in / reserve_out * PRECISION
    let price = reserve_in
        .checked_mul(U256::from(crate::PRECISION_MULTIPLIER))
        .ok_or_else(|| OrbitalError::overflow("price calculation"))?
        .checked_div(reserve_out)
        .ok_or_else(|| OrbitalError::division_by_zero("price calculation"))?;

    Ok(price)
}

/// Decompose a reserve vector into components parallel and perpendicular to 1⃗
///
/// # Arguments
/// * `reserves` - Reserve vector to decompose
///
/// # Returns
/// * `(parallel, perpendicular)` - Components parallel and perpendicular to 1⃗ = (1,1,...,1)
///
/// # Algorithm
/// For any vector r⃗, we can decompose it as:
/// r⃗ = r∥ + r⊥
///
/// where r∥ is parallel to 1⃗ and r⊥ is perpendicular
///
/// r∥ = (r⃗ · 1⃗ / N) * 1⃗
/// r⊥ = r⃗ - r∥
pub fn polar_decomposition(reserves: &[U256]) -> Result<(U256, Vec<U256>)> {
    if reserves.is_empty() {
        return Err(OrbitalError::InvalidTokenCount(0));
    }

    let n = U256::from(reserves.len());

    // Calculate r⃗ · 1⃗ = Σr_i
    let dot_product: U256 = reserves.iter().fold(U256::ZERO, |acc, &r| acc + r);

    // parallel component = (r⃗ · 1⃗) / N
    let parallel = dot_product
        .checked_div(n)
        .ok_or_else(|| OrbitalError::division_by_zero("polar decomposition"))?;

    // perpendicular components = r_i - parallel for each i
    let perpendicular: Vec<U256> = reserves
        .iter()
        .map(|&r| {
            if r >= parallel {
                r - parallel
            } else {
                U256::ZERO // Shouldn't happen in practice but be safe
            }
        })
        .collect();

    Ok((parallel, perpendicular))
}

/// Calculate the equal price point where all reserves are equal
///
/// # Arguments
/// * `radius_squared` - The R² constant
/// * `token_count` - Number of tokens in pool
///
/// # Returns
/// * Reserve amount for each token at equal price point
///
/// # Algorithm
/// At equal price point, all r_i = r
/// So: N * r² = R²
/// Therefore: r = R / sqrt(N)
pub fn calculate_equal_price_point(radius_squared: U256, token_count: usize) -> Result<U256> {
    if token_count < MIN_TOKENS {
        return Err(OrbitalError::InvalidTokenCount(token_count));
    }

    let n = U256::from(token_count);

    // r² = R² / N
    let r_squared = radius_squared
        .checked_div(n)
        .ok_or_else(|| OrbitalError::division_by_zero("equal price point"))?;

    // r = sqrt(R² / N)
    Ok(sqrt_approx(r_squared))
}

/// Calculate price impact of a trade
///
/// # Arguments
/// * `reserves_before` - Reserves before trade
/// * `reserves_after` - Reserves after trade
/// * `token_in` - Input token index
/// * `token_out` - Output token index
///
/// # Returns
/// * Price impact in basis points (e.g., 50 = 0.5%)
pub fn calculate_price_impact(
    reserves_before: &[U256],
    reserves_after: &[U256],
    token_in: usize,
    token_out: usize,
) -> Result<u32> {
    let price_before = calculate_price_sphere(reserves_before, token_in, token_out)?;
    let price_after = calculate_price_sphere(reserves_after, token_in, token_out)?;

    if price_before.is_zero() {
        return Ok(0);
    }

    // impact = |price_after - price_before| / price_before * 10000
    let price_diff = if price_after > price_before {
        price_after - price_before
    } else {
        price_before - price_after
    };

    let impact_scaled = (price_diff * U256::from(10000))
        .checked_div(price_before)
        .ok_or_else(|| OrbitalError::division_by_zero("price impact"))?;

    // Convert to u32, capping at u32::MAX
    Ok(impact_scaled.try_into().unwrap_or(u32::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_sphere_constraint_valid() {
        // 3² + 4² = 9 + 16 = 25 = 5²
        let reserves = vec![U256::from(3), U256::from(4)];
        let radius_squared = U256::from(25);
        
        assert!(verify_sphere_constraint(&reserves, radius_squared, 0).is_ok());
    }

    #[test]
    fn test_verify_sphere_constraint_invalid() {
        let reserves = vec![U256::from(3), U256::from(5)]; // 9 + 25 = 34 ≠ 25
        let radius_squared = U256::from(25);
        
        assert!(verify_sphere_constraint(&reserves, radius_squared, 0).is_err());
    }

    #[test]
    fn test_verify_sphere_constraint_tolerance() {
        // Slightly off but within 1% tolerance
        let reserves = vec![U256::from(3), U256::from(4)];
        let radius_squared = U256::from(25);
        
        // Exact match
        assert!(verify_sphere_constraint(&reserves, radius_squared, 0).is_ok());
        
        // Within 1% tolerance
        assert!(verify_sphere_constraint(&reserves, U256::from(26), 100).is_ok());
    }

    #[test]
    fn test_calculate_equal_price_point() {
        // For 4 tokens on sphere with R² = 400:
        // r = sqrt(400/4) = sqrt(100) = 10
        let result = calculate_equal_price_point(U256::from(400), 4).unwrap();
        assert_eq!(result, U256::from(10));
    }

    #[test]
    fn test_calculate_price_sphere() {
        let reserves = vec![U256::from(100), U256::from(200)];
        
        // Price of token 1 in terms of token 0 = 100/200 = 0.5
        let price = calculate_price_sphere(&reserves, 0, 1).unwrap();
        let expected = U256::from(crate::PRECISION_MULTIPLIER / 2);
        assert_eq!(price, expected);
    }

    #[test]
    fn test_polar_decomposition() {
        let reserves = vec![U256::from(10), U256::from(20), U256::from(30)];
        
        let (parallel, perpendicular) = polar_decomposition(&reserves).unwrap();
        
        // parallel = (10+20+30)/3 = 20
        assert_eq!(parallel, U256::from(20));
        
        // perpendicular = [10-20, 20-20, 30-20] = [-10, 0, 10]
        // But we use saturating sub so negative becomes 0
        assert_eq!(perpendicular[0], U256::ZERO);
        assert_eq!(perpendicular[1], U256::ZERO);
        assert_eq!(perpendicular[2], U256::from(10));
    }

    #[test]
    fn test_calculate_amount_out_simple() {
        // Simple 2D case: circle with R² = 25
        // Reserves: [3, 4] (3² + 4² = 25)
        let reserves = vec![U256::from(3), U256::from(4)];
        let radius_squared = U256::from(25);
        
        // Add 1 to token 0: new_reserve_0 = 4
        // 4² + new_reserve_1² = 25
        // new_reserve_1² = 9, so new_reserve_1 = 3
        // amount_out = 4 - 3 = 1
        let amount_out = calculate_amount_out_sphere(
            &reserves,
            0,  // token_in
            1,  // token_out
            U256::from(1),  // amount_in
            radius_squared,
        ).unwrap();
        
        assert_eq!(amount_out, U256::from(1));
    }

    #[test]
    fn test_calculate_amount_out_3d() {
        // 3D sphere: R² = 14 (approx)
        // Initial: [2, 2, 2] → 4+4+4 = 12 (approximately on sphere)
        let reserves = vec![U256::from(2), U256::from(2), U256::from(2)];
        let radius_squared = U256::from(12);
        
        let amount_out = calculate_amount_out_sphere(
            &reserves,
            0,
            1,
            U256::from(1),
            radius_squared,
        );
        
        // Should succeed and return some positive amount
        assert!(amount_out.is_ok());
        assert!(amount_out.unwrap() > U256::ZERO);
    }

    #[test]
    fn test_insufficient_liquidity() {
        let reserves = vec![U256::from(10), U256::from(10)];
        let radius_squared = U256::from(200);
        
        // Try to remove more than available
        let result = calculate_amount_out_sphere(
            &reserves,
            0,
            1,
            U256::from(1000),  // huge amount
            radius_squared,
        );
        
        assert!(result.is_err());
    }
}