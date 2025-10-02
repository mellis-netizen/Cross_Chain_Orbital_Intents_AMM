use serde::{Deserialize, Serialize};
use std::env;
use tokio::fs;
use eyre::Result;
use ethers::types::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_address: String,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub rate_limit: RateLimitConfig,
    pub chains: Vec<ChainConfig>,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u64,
    pub burst_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub intents_contract: Address,
    pub orbital_amm_contract: Address,
    pub bridge_contract: Address,
    pub confirmation_blocks: u64,
    pub gas_price_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub update_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0:8080".to_string(),
            database_url: "postgresql://localhost/intents".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            jwt_secret: "your-secret-key".to_string(),
            rate_limit: RateLimitConfig {
                requests_per_minute: 100,
                burst_size: 20,
            },
            chains: vec![
                ChainConfig {
                    chain_id: 17000, // Holesky testnet
                    name: "Holesky".to_string(),
                    rpc_url: "https://holesky.gateway.tenderly.co".to_string(),
                    intents_contract: "0x0000000000000000000000000000000000000000".parse().unwrap(),
                    orbital_amm_contract: "0x0000000000000000000000000000000000000000".parse().unwrap(),
                    bridge_contract: "0x0000000000000000000000000000000000000000".parse().unwrap(),
                    confirmation_blocks: 3,
                    gas_price_multiplier: 1.2,
                },
            ],
            metrics: MetricsConfig {
                enabled: true,
                endpoint: "/metrics".to_string(),
                update_interval_secs: 15,
            },
        }
    }
}

impl Config {
    pub async fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        if let Ok(addr) = env::var("SERVER_ADDRESS") {
            config.server_address = addr;
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }

        if let Ok(redis_url) = env::var("REDIS_URL") {
            config.redis_url = redis_url;
        }

        if let Ok(jwt_secret) = env::var("JWT_SECRET") {
            config.jwt_secret = jwt_secret;
        }

        // Rate limiting from env
        if let Ok(rpm) = env::var("RATE_LIMIT_RPM") {
            config.rate_limit.requests_per_minute = rpm.parse().unwrap_or(100);
        }

        if let Ok(burst) = env::var("RATE_LIMIT_BURST") {
            config.rate_limit.burst_size = burst.parse().unwrap_or(20);
        }

        // Chain configurations from env
        if let Ok(holesky_rpc) = env::var("HOLESKY_RPC_URL") {
            if let Some(chain) = config.chains.iter_mut().find(|c| c.chain_id == 17000) {
                chain.rpc_url = holesky_rpc;
            }
        }

        if let Ok(intents_contract) = env::var("INTENTS_CONTRACT_ADDRESS") {
            if let Ok(addr) = intents_contract.parse::<Address>() {
                if let Some(chain) = config.chains.iter_mut().find(|c| c.chain_id == 17000) {
                    chain.intents_contract = addr;
                }
            }
        }

        if let Ok(orbital_contract) = env::var("ORBITAL_AMM_CONTRACT_ADDRESS") {
            if let Ok(addr) = orbital_contract.parse::<Address>() {
                if let Some(chain) = config.chains.iter_mut().find(|c| c.chain_id == 17000) {
                    chain.orbital_amm_contract = addr;
                }
            }
        }

        Ok(config)
    }

    pub fn get_chain_config(&self, chain_id: u64) -> Option<&ChainConfig> {
        self.chains.iter().find(|c| c.chain_id == chain_id)
    }
}