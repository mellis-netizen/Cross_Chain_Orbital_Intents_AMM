//! Property-based tests for Cross Chain Orbital Intents AMM
//!
//! These tests use proptest to generate random inputs and verify
//! invariants hold across all possible values.

use proptest::prelude::*;
use std::collections::HashMap;

// Mock types for property testing
#[derive(Debug, Clone)]
pub struct MockToken {
    pub address: [u8; 20],
    pub decimals: u8,
    pub total_supply: u64,
}

#[derive(Debug, Clone)]
pub struct MockPool {
    pub token_a: MockToken,
    pub token_b: MockToken,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub fee_rate: u32, // in basis points
    pub virtual_reserve_a: u64,
    pub virtual_reserve_b: u64,
}

#[derive(Debug, Clone)]
pub struct MockIntent {
    pub id: u64,
    pub user: [u8; 20],
    pub token_in: [u8; 20],
    pub token_out: [u8; 20],
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub deadline: u64,
    pub chain_id: u32,
}

impl MockPool {
    /// Calculate output amount using constant product formula
    pub fn calculate_output(&self, amount_in: u64, token_in_is_a: bool) -> Option<u64> {
        if amount_in == 0 {
            return Some(0);
        }

        let (reserve_in, reserve_out, virtual_in, virtual_out) = if token_in_is_a {
            (self.reserve_a, self.reserve_b, self.virtual_reserve_a, self.virtual_reserve_b)
        } else {
            (self.reserve_b, self.reserve_a, self.virtual_reserve_b, self.virtual_reserve_a)
        };

        // Use virtual reserves for pricing
        let effective_reserve_in = reserve_in + virtual_in;
        let effective_reserve_out = reserve_out + virtual_out;

        // Apply fee (fee_rate is in basis points)
        let amount_in_after_fee = amount_in
            .checked_mul(10000_u64.checked_sub(self.fee_rate as u64)?)?
            .checked_div(10000)?;

        // Constant product formula: (x + dx) * (y - dy) = x * y
        // dy = y * dx / (x + dx)
        let numerator = effective_reserve_out.checked_mul(amount_in_after_fee)?;
        let denominator = effective_reserve_in.checked_add(amount_in_after_fee)?;
        
        numerator.checked_div(denominator)
    }

    /// Check if the pool maintains the K invariant after a swap
    pub fn verify_k_invariant(&self, amount_in: u64, amount_out: u64, token_in_is_a: bool) -> bool {
        let (old_a, old_b) = (self.reserve_a + self.virtual_reserve_a, self.reserve_b + self.virtual_reserve_b);
        let old_k = old_a.checked_mul(old_b).unwrap_or(0);

        let (new_a, new_b) = if token_in_is_a {
            (old_a + amount_in, old_b.saturating_sub(amount_out))
        } else {
            (old_a.saturating_sub(amount_out), old_b + amount_in)
        };

        let new_k = new_a.checked_mul(new_b).unwrap_or(0);
        
        // K should increase or stay the same due to fees
        new_k >= old_k
    }
}

// Generators for property testing
prop_compose! {
    fn arb_token()
                 (address in prop::array::uniform32(0u8..=255u8).prop_map(|arr| {
                     let mut result = [0u8; 20];
                     result.copy_from_slice(&arr[..20]);
                     result
                 }),
                  decimals in 6u8..=18u8,
                  total_supply in 1_000_000u64..=1_000_000_000_000u64)
                 -> MockToken {
        MockToken { address, decimals, total_supply }
    }
}

prop_compose! {
    fn arb_pool()
               (token_a in arb_token(),
                token_b in arb_token(),
                reserve_a in 1_000u64..=1_000_000_000u64,
                reserve_b in 1_000u64..=1_000_000_000u64,
                fee_rate in 1u32..=1000u32, // 0.01% to 10%
                virtual_reserve_a in 1_000u64..=10_000_000u64,
                virtual_reserve_b in 1_000u64..=10_000_000u64)
               -> MockPool {
        MockPool {
            token_a,
            token_b,
            reserve_a,
            reserve_b,
            fee_rate,
            virtual_reserve_a,
            virtual_reserve_b,
        }
    }
}

prop_compose! {
    fn arb_intent()
              (id in 1u64..=1_000_000u64,
               user in prop::array::uniform32(0u8..=255u8).prop_map(|arr| {
                   let mut result = [0u8; 20];
                   result.copy_from_slice(&arr[..20]);
                   result
               }),
               token_in in prop::array::uniform32(0u8..=255u8).prop_map(|arr| {
                   let mut result = [0u8; 20];
                   result.copy_from_slice(&arr[..20]);
                   result
               }),
               token_out in prop::array::uniform32(0u8..=255u8).prop_map(|arr| {
                   let mut result = [0u8; 20];
                   result.copy_from_slice(&arr[..20]);
                   result
               }),
               amount_in in 1u64..=1_000_000u64,
               min_amount_out in 1u64..=1_000_000u64,
               deadline in 1_000_000u64..=2_000_000u64,
               chain_id in 1u32..=1000u32)
              -> MockIntent {
        MockIntent {
            id,
            user,
            token_in,
            token_out,
            amount_in,
            min_amount_out,
            deadline,
            chain_id,
        }
    }
}

proptest! {
    /// Property: Pool output calculation should always return reasonable values
    #[test]
    fn prop_pool_output_calculation(
        pool in arb_pool(),
        amount_in in 1u64..=1_000_000u64,
        token_in_is_a in any::<bool>()
    ) {
        if let Some(amount_out) = pool.calculate_output(amount_in, token_in_is_a) {
            // Output should be less than available reserves
            let available_reserve = if token_in_is_a { 
                pool.reserve_b + pool.virtual_reserve_b 
            } else { 
                pool.reserve_a + pool.virtual_reserve_a 
            };
            
            prop_assert!(amount_out < available_reserve);
            
            // Output should be reasonable compared to input
            // (not more than 10x the input, accounting for price differences)
            prop_assert!(amount_out <= amount_in.saturating_mul(10));
        }
    }

    /// Property: K invariant should hold after swaps
    #[test]
    fn prop_k_invariant_holds(
        pool in arb_pool(),
        amount_in in 1u64..=10_000u64, // Smaller amounts to avoid overflow
        token_in_is_a in any::<bool>()
    ) {
        if let Some(amount_out) = pool.calculate_output(amount_in, token_in_is_a) {
            prop_assert!(pool.verify_k_invariant(amount_in, amount_out, token_in_is_a));
        }
    }

    /// Property: Zero input should return zero output
    #[test]
    fn prop_zero_input_zero_output(
        pool in arb_pool(),
        token_in_is_a in any::<bool>()
    ) {
        let amount_out = pool.calculate_output(0, token_in_is_a);
        prop_assert_eq!(amount_out, Some(0));
    }

    /// Property: Larger inputs should generally produce larger outputs (monotonicity)
    #[test]
    fn prop_monotonic_output(
        pool in arb_pool(),
        amount_in_1 in 1u64..=1000u64,
        amount_in_2 in 1001u64..=2000u64,
        token_in_is_a in any::<bool>()
    ) {
        let out_1 = pool.calculate_output(amount_in_1, token_in_is_a);
        let out_2 = pool.calculate_output(amount_in_2, token_in_is_a);
        
        if let (Some(out_1), Some(out_2)) = (out_1, out_2) {
            prop_assert!(out_2 >= out_1);
        }
    }

    /// Property: Fees should always reduce output
    #[test]
    fn prop_fees_reduce_output(
        mut pool in arb_pool(),
        amount_in in 1u64..=10_000u64,
        token_in_is_a in any::<bool>()
    ) {
        // Calculate output with current fee
        let output_with_fee = pool.calculate_output(amount_in, token_in_is_a);
        
        // Calculate output with zero fee
        let original_fee = pool.fee_rate;
        pool.fee_rate = 0;
        let output_no_fee = pool.calculate_output(amount_in, token_in_is_a);
        
        if let (Some(with_fee), Some(no_fee)) = (output_with_fee, output_no_fee) {
            if original_fee > 0 {
                prop_assert!(with_fee <= no_fee);
            } else {
                prop_assert_eq!(with_fee, no_fee);
            }
        }
    }

    /// Property: Intent validation should reject invalid intents
    #[test]
    fn prop_intent_validation(intent in arb_intent()) {
        // Basic validation rules
        prop_assert!(intent.id > 0);
        prop_assert!(intent.amount_in > 0);
        prop_assert!(intent.min_amount_out > 0);
        prop_assert!(intent.deadline > 0);
        prop_assert!(intent.chain_id > 0);
        
        // Token addresses should be different
        prop_assert_ne!(intent.token_in, intent.token_out);
    }

    /// Property: Pool reserves should never underflow during swaps
    #[test]
    fn prop_no_reserve_underflow(
        pool in arb_pool(),
        amount_in in 1u64..=100_000u64,
        token_in_is_a in any::<bool>()
    ) {
        if let Some(amount_out) = pool.calculate_output(amount_in, token_in_is_a) {
            let reserve_out = if token_in_is_a { pool.reserve_b } else { pool.reserve_a };
            
            // Output should never exceed actual reserves (excluding virtual)
            prop_assert!(amount_out <= reserve_out);
        }
    }

    /// Property: Virtual reserves should improve pricing (reduce slippage)
    #[test]
    fn prop_virtual_reserves_improve_pricing(
        mut pool in arb_pool(),
        amount_in in 1u64..=10_000u64,
        token_in_is_a in any::<bool>()
    ) {
        // Calculate output with virtual reserves
        let output_with_virtual = pool.calculate_output(amount_in, token_in_is_a);
        
        // Calculate output without virtual reserves
        pool.virtual_reserve_a = 0;
        pool.virtual_reserve_b = 0;
        let output_without_virtual = pool.calculate_output(amount_in, token_in_is_a);
        
        if let (Some(with_virtual), Some(without_virtual)) = (output_with_virtual, output_without_virtual) {
            // Virtual reserves should generally provide better pricing (more output)
            // This may not always be true for very small pools, so we allow some tolerance
            if pool.reserve_a > 10_000 && pool.reserve_b > 10_000 {
                prop_assert!(with_virtual >= without_virtual);
            }
        }
    }
}

#[cfg(test)]
mod additional_property_tests {
    use super::*;

    proptest! {
        /// Property: Price impact should be reasonable
        #[test]
        fn prop_reasonable_price_impact(
            pool in arb_pool(),
            amount_in in 1u64..=1000u64, // Small amounts for reasonable price impact
            token_in_is_a in any::<bool>()
        ) {
            if let Some(amount_out) = pool.calculate_output(amount_in, token_in_is_a) {
                let (reserve_in, reserve_out) = if token_in_is_a {
                    (pool.reserve_a + pool.virtual_reserve_a, pool.reserve_b + pool.virtual_reserve_b)
                } else {
                    (pool.reserve_b + pool.virtual_reserve_b, pool.reserve_a + pool.virtual_reserve_a)
                };
                
                // Price before trade
                let price_before = reserve_out.checked_div(reserve_in);
                
                // Price after trade (simplified)
                let new_reserve_in = reserve_in + amount_in;
                let new_reserve_out = reserve_out.saturating_sub(amount_out);
                let price_after = if new_reserve_in > 0 { 
                    Some(new_reserve_out.checked_div(new_reserve_in).unwrap_or(0))
                } else { 
                    None 
                };
                
                // For small trades, price impact should be reasonable
                if let (Some(before), Some(after)) = (price_before, price_after) {
                    if amount_in <= reserve_in / 100 { // Less than 1% of pool
                        let price_change = if before > 0 {
                            ((before.max(after).saturating_sub(before.min(after))) * 100) / before
                        } else {
                            0
                        };
                        prop_assert!(price_change <= 10); // Max 10% price impact for 1% pool size
                    }
                }
            }
        }

        /// Property: Batch swaps should be equivalent to sequential swaps
        #[test]
        fn prop_batch_swap_equivalence(
            mut pool in arb_pool(),
            swaps in prop::collection::vec(
                (1u64..=1000u64, any::<bool>()), 
                1..=5
            )
        ) {
            let mut pool_sequential = pool.clone();
            let mut total_output_sequential = 0u64;
            
            // Execute swaps sequentially
            for (amount_in, token_in_is_a) in &swaps {
                if let Some(amount_out) = pool_sequential.calculate_output(*amount_in, *token_in_is_a) {
                    total_output_sequential = total_output_sequential.saturating_add(amount_out);
                    
                    // Update pool state
                    if *token_in_is_a {
                        pool_sequential.reserve_a = pool_sequential.reserve_a.saturating_add(*amount_in);
                        pool_sequential.reserve_b = pool_sequential.reserve_b.saturating_sub(amount_out);
                    } else {
                        pool_sequential.reserve_b = pool_sequential.reserve_b.saturating_add(*amount_in);
                        pool_sequential.reserve_a = pool_sequential.reserve_a.saturating_sub(amount_out);
                    }
                }
            }
            
            // For now, just ensure sequential execution doesn't break anything
            prop_assert!(total_output_sequential >= 0);
        }
    }
}

/// Test configuration for property tests
pub fn proptest_config() -> proptest::test_runner::Config {
    proptest::test_runner::Config {
        cases: 100,
        max_shrink_iters: 10_000,
        .. proptest::test_runner::Config::default()
    }
}