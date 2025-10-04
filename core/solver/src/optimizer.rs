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
        let mut optimizer = Self {
            pools: HashMap::new(),
            bridges: Vec::new(),
        };
        
        // Load pool and bridge information from on-chain
        optimizer.load_pools_and_bridges(config).await?;
        
        Ok(optimizer)
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
        self.find_complex_cross_chain_route(intent).await
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
    
    /// Load pools and bridges information from on-chain sources
    async fn load_pools_and_bridges(&mut self, config: &SolverConfig) -> Result<()> {
        // Load pools for each supported chain
        for chain_id in &config.supported_chains {
            let pools = self.load_chain_pools(*chain_id).await?;
            self.pools.insert(*chain_id, pools);
        }
        
        // Load bridge information
        self.bridges = self.load_bridge_info(&config.supported_chains).await?;
        
        Ok(())
    }
    
    /// Load pool information for a specific chain
    async fn load_chain_pools(&self, chain_id: u64) -> Result<Vec<PoolInfo>> {
        // In production, this would query actual DEX protocols
        // For now, create mock pools with realistic data
        
        let mut pools = Vec::new();
        
        // Common token addresses (mock)
        let tokens = self.get_common_tokens(chain_id);
        
        // Create pools for all token pairs
        for (i, &token0) in tokens.iter().enumerate() {
            for &token1 in tokens.iter().skip(i + 1) {
                // Mock pool data
                pools.push(PoolInfo {
                    protocol: Protocol::OrbitalAMM,
                    address: self.generate_pool_address(token0, token1, chain_id),
                    token0,
                    token1,
                    reserve0: U256::from(1_000_000) * U256::from(10).pow(18.into()),
                    reserve1: U256::from(1_000_000) * U256::from(10).pow(18.into()),
                    fee: 3000, // 0.3%
                });
                
                // Add Uniswap V3 variant
                pools.push(PoolInfo {
                    protocol: Protocol::UniswapV3,
                    address: self.generate_pool_address(token1, token0, chain_id),
                    token0,
                    token1,
                    reserve0: U256::from(2_000_000) * U256::from(10).pow(18.into()),
                    reserve1: U256::from(2_000_000) * U256::from(10).pow(18.into()),
                    fee: 500, // 0.05%
                });
            }
        }
        
        Ok(pools)
    }
    
    /// Load bridge information for cross-chain operations
    async fn load_bridge_info(&self, supported_chains: &[u64]) -> Result<Vec<BridgeInfo>> {
        let mut bridges = Vec::new();
        
        // Create bridges between all supported chain pairs
        for (i, &source_chain) in supported_chains.iter().enumerate() {
            for &dest_chain in supported_chains.iter().skip(i + 1) {
                // Get common tokens for both chains
                let source_tokens = self.get_common_tokens(source_chain);
                let dest_tokens = self.get_common_tokens(dest_chain);
                
                let mut supported_tokens = Vec::new();
                
                // Map equivalent tokens across chains
                for (&source_token, &dest_token) in source_tokens.iter().zip(dest_tokens.iter()) {
                    supported_tokens.push((source_token, dest_token));
                }
                
                // Bridge from source to dest
                bridges.push(BridgeInfo {
                    source_chain,
                    dest_chain,
                    supported_tokens: supported_tokens.clone(),
                    fee_bps: self.get_bridge_fee_bps(source_chain, dest_chain),
                    min_time: self.get_bridge_time(source_chain, dest_chain),
                });
                
                // Bridge from dest to source
                let reverse_tokens = supported_tokens
                    .into_iter()
                    .map(|(src, dst)| (dst, src))
                    .collect();
                
                bridges.push(BridgeInfo {
                    source_chain: dest_chain,
                    dest_chain: source_chain,
                    supported_tokens: reverse_tokens,
                    fee_bps: self.get_bridge_fee_bps(dest_chain, source_chain),
                    min_time: self.get_bridge_time(dest_chain, source_chain),
                });
            }
        }
        
        Ok(bridges)
    }
    
    /// Find complex cross-chain routing with swaps on both sides
    async fn find_complex_cross_chain_route(&self, intent: &Intent) -> Result<Route> {
        // Strategy: source_token -> bridge_token (source chain) -> bridge_token (dest chain) -> dest_token
        
        let source_pools = self.pools.get(&intent.source_chain_id)
            .ok_or(SolverError::ChainNotSupported(intent.source_chain_id))?;
        
        let dest_pools = self.pools.get(&intent.dest_chain_id)
            .ok_or(SolverError::ChainNotSupported(intent.dest_chain_id))?;
        
        // Find available bridges
        let available_bridges: Vec<&BridgeInfo> = self.bridges
            .iter()
            .filter(|b| b.source_chain == intent.source_chain_id && b.dest_chain == intent.dest_chain_id)
            .collect();
        
        let mut best_route = None;
        let mut best_output = U256::zero();
        
        for bridge in available_bridges {
            for (source_bridge_token, dest_bridge_token) in &bridge.supported_tokens {
                // Route: source_token -> source_bridge_token -> dest_bridge_token -> dest_token
                
                // Step 1: Swap on source chain (if needed)
                let after_source_swap = if intent.source_token == *source_bridge_token {
                    intent.source_amount
                } else {
                    if let Some(source_pool) = self.find_direct_pool(
                        source_pools,
                        intent.source_token,
                        *source_bridge_token,
                    ) {
                        self.calculate_swap_output(
                            source_pool,
                            intent.source_token,
                            intent.source_amount,
                        )
                    } else {
                        continue; // No route available
                    }
                };
                
                // Step 2: Bridge
                let bridge_fee = (after_source_swap * U256::from(bridge.fee_bps)) / U256::from(10000);
                let after_bridge = after_source_swap.saturating_sub(bridge_fee);
                
                // Step 3: Swap on destination chain (if needed)
                let final_amount = if *dest_bridge_token == intent.dest_token {
                    after_bridge
                } else {
                    if let Some(dest_pool) = self.find_direct_pool(
                        dest_pools,
                        *dest_bridge_token,
                        intent.dest_token,
                    ) {
                        self.calculate_swap_output(dest_pool, *dest_bridge_token, after_bridge)
                    } else {
                        continue; // No route available
                    }
                };
                
                if final_amount > best_output && final_amount >= intent.min_dest_amount {
                    best_output = final_amount;
                    
                    // Build route with all hops
                    let mut hops = Vec::new();
                    
                    // Source swap hop (if needed)
                    if intent.source_token != *source_bridge_token {
                        if let Some(source_pool) = self.find_direct_pool(
                            source_pools,
                            intent.source_token,
                            *source_bridge_token,
                        ) {
                            hops.push(Hop {
                                protocol: source_pool.protocol,
                                chain_id: intent.source_chain_id,
                                pool_address: source_pool.address,
                                token_in: intent.source_token,
                                token_out: *source_bridge_token,
                                amount_in: intent.source_amount,
                                amount_out: after_source_swap,
                            });
                        }
                    }
                    
                    // Bridge hop
                    hops.push(Hop {
                        protocol: Protocol::Bridge,
                        chain_id: intent.source_chain_id, // Bridge starts on source chain
                        pool_address: Address::zero(), // Bridge doesn't have a pool address
                        token_in: *source_bridge_token,
                        token_out: *dest_bridge_token,
                        amount_in: after_source_swap,
                        amount_out: after_bridge,
                    });
                    
                    // Destination swap hop (if needed)
                    if *dest_bridge_token != intent.dest_token {
                        if let Some(dest_pool) = self.find_direct_pool(
                            dest_pools,
                            *dest_bridge_token,
                            intent.dest_token,
                        ) {
                            hops.push(Hop {
                                protocol: dest_pool.protocol,
                                chain_id: intent.dest_chain_id,
                                pool_address: dest_pool.address,
                                token_in: *dest_bridge_token,
                                token_out: intent.dest_token,
                                amount_in: after_bridge,
                                amount_out: final_amount,
                            });
                        }
                    }
                    
                    best_route = Some(Route {
                        hops,
                        estimated_gas: U256::from(500_000), // Higher gas for complex route
                        estimated_output: final_amount,
                        cross_chain: true,
                    });
                }
            }
        }
        
        best_route.ok_or(SolverError::InsufficientLiquidity)
    }
    
    /// Get common tokens for a chain (mock implementation)
    fn get_common_tokens(&self, chain_id: u64) -> Vec<Address> {
        match chain_id {
            1 => vec![
                // Ethereum mainnet tokens (mock addresses)
                Address::from_low_u64_be(0x1001), // WETH
                Address::from_low_u64_be(0x1002), // USDC
                Address::from_low_u64_be(0x1003), // USDT
                Address::from_low_u64_be(0x1004), // DAI
            ],
            137 => vec![
                // Polygon tokens (mock addresses)
                Address::from_low_u64_be(0x2001), // WMATIC
                Address::from_low_u64_be(0x2002), // USDC
                Address::from_low_u64_be(0x2003), // USDT
                Address::from_low_u64_be(0x2004), // DAI
            ],
            42161 => vec![
                // Arbitrum tokens (mock addresses)
                Address::from_low_u64_be(0x3001), // WETH
                Address::from_low_u64_be(0x3002), // USDC
                Address::from_low_u64_be(0x3003), // USDT
                Address::from_low_u64_be(0x3004), // DAI
            ],
            _ => vec![
                // Default tokens
                Address::from_low_u64_be(0x9001),
                Address::from_low_u64_be(0x9002),
            ],
        }
    }
    
    /// Generate deterministic pool address from tokens
    fn generate_pool_address(&self, token0: Address, token1: Address, chain_id: u64) -> Address {
        use ethers::core::utils::keccak256;
        
        let mut data = Vec::new();
        data.extend_from_slice(token0.as_bytes());
        data.extend_from_slice(token1.as_bytes());
        data.extend_from_slice(&chain_id.to_le_bytes());
        
        let hash = keccak256(data);
        Address::from_slice(&hash[12..32])
    }
    
    /// Get bridge fee in basis points
    fn get_bridge_fee_bps(&self, source_chain: u64, dest_chain: u64) -> u16 {
        match (source_chain, dest_chain) {
            (1, 137) | (137, 1) => 50,   // 0.5% for Ethereum <-> Polygon
            (1, 42161) | (42161, 1) => 25, // 0.25% for Ethereum <-> Arbitrum
            (137, 42161) | (42161, 137) => 30, // 0.3% for Polygon <-> Arbitrum
            _ => 100, // 1% for other bridges
        }
    }
    
    /// Get bridge time in seconds
    fn get_bridge_time(&self, source_chain: u64, dest_chain: u64) -> u64 {
        match (source_chain, dest_chain) {
            (1, 137) | (137, 1) => 300,   // 5 minutes for Ethereum <-> Polygon
            (1, 42161) | (42161, 1) => 600, // 10 minutes for Ethereum <-> Arbitrum
            (137, 42161) | (42161, 137) => 180, // 3 minutes for Polygon <-> Arbitrum
            _ => 900, // 15 minutes for other bridges
        }
    }
}