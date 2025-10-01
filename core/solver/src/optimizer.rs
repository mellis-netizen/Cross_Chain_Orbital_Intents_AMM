use crate::{Result, SolverError, SolverConfig};
use ethers::types::{Address, U256};
use intents_engine::intent::Intent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub hops: Vec<Hop>,
    pub estimated_gas: U256,
    pub estimated_output: U256,
    pub cross_chain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hop {
    pub protocol: Protocol,
    pub chain_id: u64,
    pub pool_address: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub amount_out: U256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    OrbitalAMM,
    UniswapV3,
    SushiSwap,
    Curve,
    Bridge,
}

pub struct RouteOptimizer {
    pools: HashMap<u64, Vec<PoolInfo>>,
    bridges: Vec<BridgeInfo>,
}

#[derive(Clone)]
struct PoolInfo {
    protocol: Protocol,
    address: Address,
    token0: Address,
    token1: Address,
    reserve0: U256,
    reserve1: U256,
    fee: u16,
}

#[derive(Clone)]
struct BridgeInfo {
    source_chain: u64,
    dest_chain: u64,
    supported_tokens: Vec<(Address, Address)>, // (source_token, dest_token)
    fee_bps: u16,
    min_time: u64,
}

impl RouteOptimizer {
    pub async fn new(config: &SolverConfig) -> Result<Self> {
        // TODO: Load pool and bridge information from on-chain
        Ok(Self {
            pools: HashMap::new(),
            bridges: Vec::new(),
        })
    }
    
    pub async fn find_best_route(&self, intent: &Intent) -> Result<Route> {
        if intent.source_chain_id == intent.dest_chain_id {
            self.find_single_chain_route(intent).await
        } else {
            self.find_cross_chain_route(intent).await
        }
    }
    
    async fn find_single_chain_route(&self, intent: &Intent) -> Result<Route> {
        let pools = self.pools.get(&intent.source_chain_id)
            .ok_or(SolverError::ChainNotSupported(intent.source_chain_id))?;
        
        // Find direct swap
        if let Some(pool) = self.find_direct_pool(
            pools,
            intent.source_token,
            intent.dest_token,
        ) {
            let amount_out = self.calculate_swap_output(
                pool,
                intent.source_token,
                intent.source_amount,
            );
            
            if amount_out >= intent.min_dest_amount {
                return Ok(Route {
                    hops: vec![Hop {
                        protocol: pool.protocol,
                        chain_id: intent.source_chain_id,
                        pool_address: pool.address,
                        token_in: intent.source_token,
                        token_out: intent.dest_token,
                        amount_in: intent.source_amount,
                        amount_out,
                    }],
                    estimated_gas: U256::from(150000),
                    estimated_output: amount_out,
                    cross_chain: false,
                });
            }
        }
        
        // Find multi-hop route
        self.find_multi_hop_route(intent, pools).await
    }
    
    async fn find_cross_chain_route(&self, intent: &Intent) -> Result<Route> {
        // Find best bridge
        let bridge = self.find_best_bridge(
            intent.source_chain_id,
            intent.dest_chain_id,
            intent.source_token,
            intent.dest_token,
        )?;
        
        // Calculate bridge output
        let bridge_fee = intent.source_amount * U256::from(bridge.fee_bps) / U256::from(10000);
        let amount_after_bridge = intent.source_amount - bridge_fee;
        
        // Check if direct bridge satisfies intent
        if intent.source_token == intent.dest_token {
            if amount_after_bridge >= intent.min_dest_amount {
                return Ok(Route {
                    hops: vec![Hop {
                        protocol: Protocol::Bridge,
                        chain_id: intent.source_chain_id,
                        pool_address: Address::zero(), // Bridge address
                        token_in: intent.source_token,
                        token_out: intent.dest_token,
                        amount_in: intent.source_amount,
                        amount_out: amount_after_bridge,
                    }],
                    estimated_gas: U256::from(300000),
                    estimated_output: amount_after_bridge,
                    cross_chain: true,
                });
            }
        }
        
        // Find swap routes on both chains
        // TODO: Implement cross-chain swap routing
        Err(SolverError::InsufficientLiquidity)
    }
    
    async fn find_multi_hop_route(
        &self,
        intent: &Intent,
        pools: &[PoolInfo],
    ) -> Result<Route> {
        // Simple 2-hop routing through common base tokens
        let base_tokens = vec![
            // WETH, USDC, USDT, DAI addresses
        ];
        
        let mut best_route = None;
        let mut best_output = U256::zero();
        
        for base_token in base_tokens {
            // Find source -> base pool
            if let Some(pool1) = self.find_direct_pool(pools, intent.source_token, base_token) {
                let mid_amount = self.calculate_swap_output(
                    pool1,
                    intent.source_token,
                    intent.source_amount,
                );
                
                // Find base -> dest pool
                if let Some(pool2) = self.find_direct_pool(pools, base_token, intent.dest_token) {
                    let final_amount = self.calculate_swap_output(pool2, base_token, mid_amount);
                    
                    if final_amount > best_output && final_amount >= intent.min_dest_amount {
                        best_output = final_amount;
                        best_route = Some(Route {
                            hops: vec![
                                Hop {
                                    protocol: pool1.protocol,
                                    chain_id: intent.source_chain_id,
                                    pool_address: pool1.address,
                                    token_in: intent.source_token,
                                    token_out: base_token,
                                    amount_in: intent.source_amount,
                                    amount_out: mid_amount,
                                },
                                Hop {
                                    protocol: pool2.protocol,
                                    chain_id: intent.source_chain_id,
                                    pool_address: pool2.address,
                                    token_in: base_token,
                                    token_out: intent.dest_token,
                                    amount_in: mid_amount,
                                    amount_out: final_amount,
                                },
                            ],
                            estimated_gas: U256::from(250000),
                            estimated_output: final_amount,
                            cross_chain: false,
                        });
                    }
                }
            }
        }
        
        best_route.ok_or(SolverError::InsufficientLiquidity)
    }
    
    fn find_direct_pool(
        &self,
        pools: &[PoolInfo],
        token0: Address,
        token1: Address,
    ) -> Option<&PoolInfo> {
        pools.iter().find(|p| {
            (p.token0 == token0 && p.token1 == token1) ||
            (p.token0 == token1 && p.token1 == token0)
        })
    }
    
    fn find_best_bridge(
        &self,
        source_chain: u64,
        dest_chain: u64,
        source_token: Address,
        dest_token: Address,
    ) -> Result<&BridgeInfo> {
        self.bridges.iter()
            .find(|b| {
                b.source_chain == source_chain &&
                b.dest_chain == dest_chain &&
                b.supported_tokens.iter().any(|(st, dt)| {
                    *st == source_token && *dt == dest_token
                })
            })
            .ok_or(SolverError::ChainNotSupported(source_chain))
    }
    
    fn calculate_swap_output(
        &self,
        pool: &PoolInfo,
        token_in: Address,
        amount_in: U256,
    ) -> U256 {
        let (reserve_in, reserve_out) = if pool.token0 == token_in {
            (pool.reserve0, pool.reserve1)
        } else {
            (pool.reserve1, pool.reserve0)
        };
        
        // Apply fee
        let amount_in_with_fee = amount_in * U256::from(10000 - pool.fee) / U256::from(10000);
        
        // x * y = k formula
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in + amount_in_with_fee;
        
        numerator / denominator
    }
    
    pub async fn calculate_profit(
        &self,
        route: &Route,
        intent: &Intent,
    ) -> Result<(U256, U256)> {
        let output = route.estimated_output;
        let profit = if output > intent.min_dest_amount {
            output - intent.min_dest_amount
        } else {
            U256::zero()
        };
        
        Ok((output, profit))
    }
}