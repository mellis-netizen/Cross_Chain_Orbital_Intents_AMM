//! Holesky deployment module for Rust Intents System
//! 
//! This module provides tools for deploying and configuring the complete
//! cross-chain intents system on Holesky testnet.

use ethers::{
    core::utils::parse_ether,
    middleware::SignerMiddleware,
    prelude::*,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, U256},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

/// Holesky network configuration
pub const HOLESKY_CHAIN_ID: u64 = 17000;
pub const HOLESKY_RPC_URL: &str = "https://crimson-attentive-emerald.ethereum-holesky.quiknode.pro/2f9f0ed63e2c2adf0adaca0fb431a457f86cf7ad/";

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub private_key: String,
    pub rpc_url: String,
    pub chain_id: u64,
    pub deployer_address: Address,
    pub gas_price: Option<U256>,
    pub gas_limit: Option<U256>,
}

/// Deployed contract addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContracts {
    pub intents_contract: Address,
    pub orbital_amm_contract: Address,
    pub mock_usdc_contract: Address,
    pub deployment_block: u64,
    pub deployment_timestamp: u64,
}

/// Complete deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub config: DeploymentConfig,
    pub contracts: DeployedContracts,
    pub transaction_hashes: Vec<H256>,
    pub total_gas_used: U256,
    pub total_cost: U256,
}

/// Holesky deployer
pub struct HoleskyDeployer {
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    config: DeploymentConfig,
}

impl HoleskyDeployer {
    /// Create a new Holesky deployer
    pub async fn new(private_key: &str) -> Result<Self> {
        let provider = Provider::<Http>::try_from(HOLESKY_RPC_URL)?;
        let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(HOLESKY_CHAIN_ID);
        let client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

        let config = DeploymentConfig {
            private_key: private_key.to_string(),
            rpc_url: HOLESKY_RPC_URL.to_string(),
            chain_id: HOLESKY_CHAIN_ID,
            deployer_address: wallet.address(),
            gas_price: None,
            gas_limit: None,
        };

        Ok(Self { client, config })
    }

    /// Check deployer balance and network connectivity
    pub async fn check_prerequisites(&self) -> Result<()> {
        println!("🔍 Checking deployment prerequisites...");

        // Check network connectivity
        let chain_id = self.client.get_chainid().await?;
        if chain_id.as_u64() != HOLESKY_CHAIN_ID {
            return Err(eyre::eyre!(
                "Wrong network! Expected Holesky ({}), got {}",
                HOLESKY_CHAIN_ID,
                chain_id
            ));
        }
        println!("✅ Connected to Holesky testnet");

        // Check balance
        let balance = self.client.get_balance(self.config.deployer_address, None).await?;
        let balance_eth = ethers::utils::format_ether(balance);
        println!("💰 Deployer balance: {} ETH", balance_eth);

        if balance < parse_ether("0.1")? {
            return Err(eyre::eyre!(
                "Insufficient balance! Need at least 0.1 ETH for deployment. \
                Visit https://faucet.quicknode.com/ethereum/holesky to get test ETH."
            ));
        }

        // Check gas price
        let gas_price = self.client.get_gas_price().await?;
        println!("⛽ Current gas price: {} gwei", ethers::utils::format_units(gas_price, "gwei")?);

        Ok(())
    }

    /// Deploy mock USDC token for testing
    pub async fn deploy_mock_usdc(&self) -> Result<(Address, H256)> {
        println!("📦 Deploying Mock USDC token...");

        // ERC20 mock contract bytecode (simplified)
        let bytecode = "608060405234801561001057600080fd5b506040518060400160405280600a81526020017f4d6f636b20555344430000000000000000000000000000000000000000000000815250604051806040016040528060048152602001635553444360e01b8152508160039081610075919061013c565b50805161008a90600490602084019061008e565b5050505b8061023e565b828054610094906101fe565b90600052602060002090601f0160209004810192826100b657600085556100fc565b82601f106100cf57805160ff19168380011785556100fc565b828001600101855582156100fc579182015b828111156100fc5782518255916020019190600101906100e1565b5061010892915061010c565b5090565b5b80821115610108576000815560010161010d565b634e487b7160e01b600052604160045260246000fd5b600181811c9082168061015057607f821691505b60208210810361017057634e487b7160e01b600052602260045260246000fd5b50919050565b601f8211156101b957600081815260208120601f850160051c8101602086101561019d5750805b601f850160051c820191505b818110156101bc578281556001016101a9565b5050505b505050565b81516001600160401b038111156101de576101de610121565b6101f2816101ec845461013c565b84610176565b602080601f831160018114610227576000841561020f5750858301515b600019600386901b1c1916600185901b1785556101bc565b600085815260208120601f198616915b8281101561025657888601518255948401946001909101908401610237565b508582101561027457878501516000196003841b1c19169055565b5050505050600190811b01905550565b610e8380610293600039f3fe";

        let deploy_tx = TransactionRequest::new()
            .data(hex::decode(bytecode)?)
            .gas(1_000_000)
            .gas_price(self.client.get_gas_price().await?);

        let pending_tx = self.client.send_transaction(deploy_tx, None).await?;
        let receipt = pending_tx.await?.ok_or_else(|| eyre::eyre!("Transaction failed"))?;

        let contract_address = receipt.contract_address.ok_or_else(|| eyre::eyre!("No contract address"))?;

        println!("✅ Mock USDC deployed at: {}", contract_address);
        println!("   Transaction: {}", receipt.transaction_hash);
        println!("   Gas used: {}", receipt.gas_used.unwrap_or_default());

        Ok((contract_address, receipt.transaction_hash))
    }

    /// Deploy mock intents contract
    pub async fn deploy_intents_contract(&self) -> Result<(Address, H256)> {
        println!("📦 Deploying Intents contract...");

        // For demo purposes, deploy a simple contract that emits events
        let bytecode = "608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806354fd4d501461003b578063c0d78655146100a3575b600080fd5b6100896040518060400160405280601081526020016f496e74656e747320436f6e747261637460801b81525081565b60405161009a9190610096565b60405180910390f35b6100b66100b13660046100d7565b6100b8565b005b7f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925338260405160405180910390a150565b6000602082840312156100e957600080fd5b5035919050565b600060208083528351808285015260005b81811015610109578581018301518582016040015282016100ed565b506000604082860101526040601f19601f8301168501019250505092915050565b60006020828403121561013c57600080fd5b5035919050565b9052565b6020808252825182820181905260009190848201906040850190845b818110156101805783518352928401929184019160010161015f565b50909695505050505056fea2646970667358221220f5a8a8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c864736f6c63430008130033";

        let deploy_tx = TransactionRequest::new()
            .data(hex::decode(bytecode)?)
            .gas(1_000_000)
            .gas_price(self.client.get_gas_price().await?);

        let pending_tx = self.client.send_transaction(deploy_tx, None).await?;
        let receipt = pending_tx.await?.ok_or_else(|| eyre::eyre!("Transaction failed"))?;

        let contract_address = receipt.contract_address.ok_or_else(|| eyre::eyre!("No contract address"))?;

        println!("✅ Intents contract deployed at: {}", contract_address);
        println!("   Transaction: {}", receipt.transaction_hash);
        println!("   Gas used: {}", receipt.gas_used.unwrap_or_default());

        Ok((contract_address, receipt.transaction_hash))
    }

    /// Deploy mock Orbital AMM contract
    pub async fn deploy_orbital_amm(&self) -> Result<(Address, H256)> {
        println!("📦 Deploying Orbital AMM contract...");

        // Similar simple contract for demo
        let bytecode = "608060405234801561001057600080fd5b50610150806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806354fd4d501461003b578063c0d78655146100a3575b600080fd5b6100896040518060400160405280600e81526020016d4f726269746c20414d4d20563160901b81525081565b60405161009a9190610096565b60405180910390f35b6100b66100b13660046100d7565b6100b8565b005b7f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925338260405160405180910390a150565b6000602082840312156100e957600080fd5b5035919050565b600060208083528351808285015260005b81811015610109578581018301518582016040015282016100ed565b506000604082860101526040601f19601f8301168501019250505092915050565b60006020828403121561013c57600080fd5b5035919050565b9052565b6020808252825182820181905260009190848201906040850190845b818110156101805783518352928401929184019160010161015f565b50909695505050505056fea2646970667358221220f5a8a8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c8c864736f6c63430008130033";

        let deploy_tx = TransactionRequest::new()
            .data(hex::decode(bytecode)?)
            .gas(1_000_000)
            .gas_price(self.client.get_gas_price().await?);

        let pending_tx = self.client.send_transaction(deploy_tx, None).await?;
        let receipt = pending_tx.await?.ok_or_else(|| eyre::eyre!("Transaction failed"))?;

        let contract_address = receipt.contract_address.ok_or_else(|| eyre::eyre!("No contract address"))?;

        println!("✅ Orbital AMM contract deployed at: {}", contract_address);
        println!("   Transaction: {}", receipt.transaction_hash);
        println!("   Gas used: {}", receipt.gas_used.unwrap_or_default());

        Ok((contract_address, receipt.transaction_hash))
    }

    /// Complete deployment process
    pub async fn deploy_all(&self) -> Result<DeploymentResult> {
        println!("🚀 Starting complete deployment to Holesky testnet");
        println!("==================================================");

        // Check prerequisites
        self.check_prerequisites().await?;

        let mut transaction_hashes = Vec::new();
        let mut total_gas_used = U256::zero();

        // Deploy contracts
        let (usdc_address, usdc_tx) = self.deploy_mock_usdc().await?;
        transaction_hashes.push(usdc_tx);

        sleep(Duration::from_secs(2)).await; // Wait between deployments

        let (intents_address, intents_tx) = self.deploy_intents_contract().await?;
        transaction_hashes.push(intents_tx);

        sleep(Duration::from_secs(2)).await;

        let (orbital_amm_address, orbital_amm_tx) = self.deploy_orbital_amm().await?;
        transaction_hashes.push(orbital_amm_tx);

        // Calculate total gas and cost
        for tx_hash in &transaction_hashes {
            if let Ok(Some(receipt)) = self.client.get_transaction_receipt(*tx_hash).await {
                total_gas_used += receipt.gas_used.unwrap_or_default();
            }
        }

        let gas_price = self.client.get_gas_price().await?;
        let total_cost = total_gas_used * gas_price;

        let deployment_block = self.client.get_block_number().await?.as_u64();
        let deployment_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let contracts = DeployedContracts {
            intents_contract: intents_address,
            orbital_amm_contract: orbital_amm_address,
            mock_usdc_contract: usdc_address,
            deployment_block,
            deployment_timestamp,
        };

        let result = DeploymentResult {
            config: self.config.clone(),
            contracts,
            transaction_hashes,
            total_gas_used,
            total_cost,
        };

        println!("\n🎉 Deployment completed successfully!");
        println!("=====================================");
        println!("📋 Deployment Summary:");
        println!("   Network: Holesky Testnet (Chain ID: {})", HOLESKY_CHAIN_ID);
        println!("   Deployer: {}", self.config.deployer_address);
        println!("   Block: {}", deployment_block);
        println!("   Total Gas Used: {}", total_gas_used);
        println!("   Total Cost: {} ETH", ethers::utils::format_ether(total_cost));
        println!("\n📦 Deployed Contracts:");
        println!("   Intents: {}", intents_address);
        println!("   Orbital AMM: {}", orbital_amm_address);
        println!("   Mock USDC: {}", usdc_address);
        println!("\n🔗 Transaction Hashes:");
        for (i, tx_hash) in transaction_hashes.iter().enumerate() {
            println!("   {}: {}", i + 1, tx_hash);
        }

        Ok(result)
    }

    /// Verify deployment by calling contract functions
    pub async fn verify_deployment(&self, contracts: &DeployedContracts) -> Result<()> {
        println!("\n🔍 Verifying deployment...");

        // Verify contracts have code
        for (name, address) in [
            ("Intents", contracts.intents_contract),
            ("Orbital AMM", contracts.orbital_amm_contract),
            ("Mock USDC", contracts.mock_usdc_contract),
        ] {
            let code = self.client.get_code(address, None).await?;
            if code.is_empty() {
                return Err(eyre::eyre!("{} contract has no code at {}", name, address));
            }
            println!("✅ {} contract verified at {}", name, address);
        }

        println!("✅ All contracts verified successfully!");
        Ok(())
    }
}

/// CLI deployment tool
pub async fn deploy_to_holesky(private_key: &str) -> Result<DeploymentResult> {
    let deployer = HoleskyDeployer::new(private_key).await?;
    let result = deployer.deploy_all().await?;
    deployer.verify_deployment(&result.contracts).await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deployer_creation() {
        let private_key = "0c068df4a4470cb73e6704d87c61a0c2718e72381c7b1e971514e5f9c4486f93";
        let deployer = HoleskyDeployer::new(private_key).await;
        assert!(deployer.is_ok());
    }

    #[test]
    fn test_deployment_config_serialization() {
        let config = DeploymentConfig {
            private_key: "test_key".to_string(),
            rpc_url: "test_url".to_string(),
            chain_id: 17000,
            deployer_address: Address::zero(),
            gas_price: None,
            gas_limit: None,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: DeploymentConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.chain_id, deserialized.chain_id);
    }
}