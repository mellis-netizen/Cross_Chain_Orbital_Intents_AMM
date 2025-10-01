//! Utility functions for orbital math calculations

use alloy_primitives::U256;
use crate::error::{OrbitalError, Result};

/// Calculate the power of a U256 value
/// This is a simplified implementation - production should use optimized algorithms
pub fn pow(base: U256, exp: u32) -> Result<U256> {
    if exp == 0 {
        return Ok(U256::from(1));
    }
    
    if exp == 1 {
        return Ok(base);
    }
    
    if exp == 2 {
        return base
            .checked_mul(base)
            .ok_or_else(|| OrbitalError::overflow("power calculation"));
    }
    
    let mut result = U256::from(1);
    let mut b = base;
    let mut e = exp;
    
    while e > 0 {
        if e % 2 == 1 {
            result = result
                .checked_mul(b)
                .ok_or_else(|| OrbitalError::overflow("power calculation"))?;
        }
        b = b
            .checked_mul(b)
            .ok_or_else(|| OrbitalError::overflow("power calculation"))?;
        e /= 2;
    }
    
    Ok(result)
}

/// Calculate absolute value for U256 (identity function since U256 is unsigned)
#[inline]
pub fn abs(value: U256) -> U256 {
    value
}

/// Calculate the nth root of a value approximately
/// This is a placeholder - production needs precise nth root
pub fn nth_root_approx(value: U256, n: u32) -> Result<U256> {
    if n == 0 {
        return Err(OrbitalError::division_by_zero("nth root with n=0"));
    }
    
    if n == 1 {
        return Ok(value);
    }
    
    if n == 2 {
        return Ok(crate::types::sqrt_approx(value));
    }
    
    if value.is_zero() {
        return Ok(U256::ZERO);
    }
    
    // Newton's method for nth root
    // x_{k+1} = ((n-1)*x_k + value/x_k^(n-1)) / n
    let mut x = value / U256::from(n);
    
    for _ in 0..50 {
        let x_pow_n_minus_1 = pow(x, n - 1)?;
        
        if x_pow_n_minus_1.is_zero() {
            break;
        }
        
        let numerator = x
            .checked_mul(U256::from(n - 1))
            .ok_or_else(|| OrbitalError::overflow("nth root"))?
            .checked_add(value / x_pow_n_minus_1)
            .ok_or_else(|| OrbitalError::overflow("nth root"))?;
            
        let x_new = numerator / U256::from(n);
        
        // Check convergence
        if x_new == x || (x_new < x && x - x_new <= U256::from(1)) {
            break;
        }
        
        x = x_new;
    }
    
    Ok(x)
}

/// Linear interpolation between two values
pub fn lerp(a: U256, b: U256, t: U256, scale: U256) -> Result<U256> {
    if scale.is_zero() {
        return Err(OrbitalError::division_by_zero("lerp scale"));
    }
    
    // result = a + (b - a) * t / scale
    if b >= a {
        let diff = b - a;
        let scaled = diff
            .checked_mul(t)
            .ok_or_else(|| OrbitalError::overflow("lerp"))?;
        let delta = scaled / scale;
        a.checked_add(delta)
            .ok_or_else(|| OrbitalError::overflow("lerp"))
    } else {
        let diff = a - b;
        let scaled = diff
            .checked_mul(t)
            .ok_or_else(|| OrbitalError::overflow("lerp"))?;
        let delta = scaled / scale;
        a.checked_sub(delta)
            .ok_or_else(|| OrbitalError::underflow("lerp"))
    }
}

/// Calculate percentage of a value (basis points)
pub fn apply_percentage(value: U256, percentage_bp: u32) -> Result<U256> {
    value
        .checked_mul(U256::from(percentage_bp))
        .ok_or_else(|| OrbitalError::overflow("percentage calculation"))?
        .checked_div(U256::from(10000))
        .ok_or_else(|| OrbitalError::division_by_zero("percentage calculation"))
}

/// Check if two U256 values are approximately equal within tolerance (basis points)
pub fn approx_eq(a: U256, b: U256, tolerance_bp: u32) -> bool {
    if a == b {
        return true;
    }
    
    let (larger, smaller) = if a > b { (a, b) } else { (b, a) };
    let diff = larger - smaller;
    
    // Calculate tolerance as percentage of larger value
    let tolerance = match larger.checked_mul(U256::from(tolerance_bp)) {
        Some(t) => t / U256::from(10000),
        None => return false,
    };
    
    diff <= tolerance
}

/// Clamp a value between min and max
pub fn clamp(value: U256, min: U256, max: U256) -> U256 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Calculate the dot product of two vectors
pub fn dot_product(a: &[U256], b: &[U256]) -> Result<U256> {
    if a.len() != b.len() {
        return Err(OrbitalError::invalid_param(
            "vectors",
            "vectors must have same length",
        ));
    }
    
    a.iter()
        .zip(b.iter())
        .try_fold(U256::ZERO, |acc, (&ai, &bi)| {
            let prod = ai
                .checked_mul(bi)
                .ok_or_else(|| OrbitalError::overflow("dot product"))?;
            acc.checked_add(prod)
                .ok_or_else(|| OrbitalError::overflow("dot product"))
        })
}

/// Calculate the L2 norm (Euclidean length) of a vector
pub fn l2_norm(vec: &[U256]) -> Result<U256> {
    let sum_of_squares = vec
        .iter()
        .try_fold(U256::ZERO, |acc, &v| {
            let sq = v
                .checked_mul(v)
                .ok_or_else(|| OrbitalError::overflow("L2 norm"))?;
            acc.checked_add(sq)
                .ok_or_else(|| OrbitalError::overflow("L2 norm"))
        })?;
    
    Ok(crate::types::sqrt_approx(sum_of_squares))
}

/// Calculate the sum of a vector
pub fn sum(vec: &[U256]) -> Result<U256> {
    vec.iter()
        .try_fold(U256::ZERO, |acc, &v| {
            acc.checked_add(v)
                .ok_or_else(|| OrbitalError::overflow("sum"))
        })
}

/// Calculate weighted average
pub fn weighted_average(values: &[U256], weights: &[U256]) -> Result<U256> {
    if values.len() != weights.len() {
        return Err(OrbitalError::invalid_param(
            "weighted average",
            "values and weights must have same length",
        ));
    }
    
    let weighted_sum = values
        .iter()
        .zip(weights.iter())
        .try_fold(U256::ZERO, |acc, (&v, &w)| {
            let prod = v
                .checked_mul(w)
                .ok_or_else(|| OrbitalError::overflow("weighted average"))?;
            acc.checked_add(prod)
                .ok_or_else(|| OrbitalError::overflow("weighted average"))
        })?;
    
    let total_weight = sum(weights)?;
    
    if total_weight.is_zero() {
        return Err(OrbitalError::division_by_zero("weighted average"));
    }
    
    weighted_sum
        .checked_div(total_weight)
        .ok_or_else(|| OrbitalError::division_by_zero("weighted average"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pow() {
        assert_eq!(pow(U256::from(2), 0).unwrap(), U256::from(1));
        assert_eq!(pow(U256::from(2), 1).unwrap(), U256::from(2));
        assert_eq!(pow(U256::from(2), 2).unwrap(), U256::from(4));
        assert_eq!(pow(U256::from(2), 3).unwrap(), U256::from(8));
        assert_eq!(pow(U256::from(3), 4).unwrap(), U256::from(81));
    }

    #[test]
    fn test_nth_root_approx() {
        assert_eq!(nth_root_approx(U256::from(8), 3).unwrap(), U256::from(2));
        assert_eq!(nth_root_approx(U256::from(16), 4).unwrap(), U256::from(2));
        assert_eq!(nth_root_approx(U256::from(100), 2).unwrap(), U256::from(10));
    }

    #[test]
    fn test_lerp() {
        let result = lerp(
            U256::from(0),
            U256::from(100),
            U256::from(50),
            U256::from(100),
        ).unwrap();
        assert_eq!(result, U256::from(50));
        
        let result = lerp(
            U256::from(100),
            U256::from(200),
            U256::from(25),
            U256::from(100),
        ).unwrap();
        assert_eq!(result, U256::from(125));
    }

    #[test]
    fn test_apply_percentage() {
        // 50% of 100 = 50
        let result = apply_percentage(U256::from(100), 5000).unwrap();
        assert_eq!(result, U256::from(50));
        
        // 1% of 1000 = 10
        let result = apply_percentage(U256::from(1000), 100).unwrap();
        assert_eq!(result, U256::from(10));
    }

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(U256::from(100), U256::from(100), 0));
        assert!(approx_eq(U256::from(100), U256::from(101), 100)); // Within 1%
        assert!(!approx_eq(U256::from(100), U256::from(110), 100)); // Not within 1%
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(U256::from(5), U256::from(10), U256::from(20)), U256::from(10));
        assert_eq!(clamp(U256::from(15), U256::from(10), U256::from(20)), U256::from(15));
        assert_eq!(clamp(U256::from(25), U256::from(10), U256::from(20)), U256::from(20));
    }

    #[test]
    fn test_dot_product() {
        let a = vec![U256::from(1), U256::from(2), U256::from(3)];
        let b = vec![U256::from(4), U256::from(5), U256::from(6)];
        
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        let result = dot_product(&a, &b).unwrap();
        assert_eq!(result, U256::from(32));
    }

    #[test]
    fn test_l2_norm() {
        let vec = vec![U256::from(3), U256::from(4)];
        // sqrt(3² + 4²) = sqrt(25) = 5
        let result = l2_norm(&vec).unwrap();
        assert_eq!(result, U256::from(5));
    }

    #[test]
    fn test_sum() {
        let vec = vec![U256::from(1), U256::from(2), U256::from(3), U256::from(4)];
        let result = sum(&vec).unwrap();
        assert_eq!(result, U256::from(10));
    }

    #[test]
    fn test_weighted_average() {
        let values = vec![U256::from(10), U256::from(20), U256::from(30)];
        let weights = vec![U256::from(1), U256::from(2), U256::from(3)];
        
        // (10*1 + 20*2 + 30*3) / (1+2+3) = 140/6 ≈ 23
        let result = weighted_average(&values, &weights).unwrap();
        assert_eq!(result, U256::from(23));
    }
}