//! Holesky deployment binary for Rust Intents System
//! 
//! This binary deploys the complete cross-chain intents system to Holesky testnet
//! and provides real-time monitoring of the deployment process.

use clap::{App, Arg};
use eyre::Result;
use intents_system::deployment::{deploy_to_holesky, DeploymentResult};
use serde_json;
use std::{fs, path::Path};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new("Rust Intents Holesky Deployer")
        .version("1.0.0")
        .author("Rust Intents Team")
        .about("Deploy the complete cross-chain intents system to Holesky testnet")
        .arg(
            Arg::with_name("private-key")
                .long("private-key")
                .value_name("PRIVATE_KEY")
                .help("Private key for deployment (without 0x prefix)")
                .required(true),
        )
        .arg(
            Arg::with_name("output-dir")
                .long("output-dir")
                .value_name("DIR")
                .help("Output directory for deployment artifacts")
                .default_value("deployments/holesky"),
        )
        .arg(
            Arg::with_name("verify")
                .long("verify")
                .help("Verify deployment after completion")
                .takes_value(false),
        )
        .get_matches();

    let private_key = matches.value_of("private-key").unwrap();
    let output_dir = matches.value_of("output-dir").unwrap();
    let verify = matches.is_present("verify");

    // Ensure private key has 0x prefix
    let private_key = if private_key.starts_with("0x") {
        private_key.to_string()
    } else {
        format!("0x{}", private_key)
    };

    println!("🚀 Rust Intents System - Holesky Deployment");
    println!("============================================");
    println!();

    // Create output directory
    fs::create_dir_all(output_dir)?;
    println!("📁 Output directory: {}", output_dir);

    // Deploy system
    println!("🔄 Starting deployment...");
    let deployment_result = deploy_to_holesky(&private_key).await?;

    // Save deployment artifacts
    save_deployment_artifacts(&deployment_result, output_dir).await?;

    // Create solver configuration
    create_solver_config(&deployment_result, output_dir).await?;

    // Create monitoring dashboard
    create_monitoring_dashboard(&deployment_result, output_dir).await?;

    // Verification
    if verify {
        verify_deployment(&deployment_result).await?;
    }

    println!("\n🎉 Deployment completed successfully!");
    println!("====================================");
    println!();
    println!("📋 Next Steps:");
    println!("  1. Run the solver: cargo run --bin solver -- --config {}/solver_config.json", output_dir);
    println!("  2. Start demo: ./scripts/demo_holesky.sh");
    println!("  3. View dashboard: open {}/dashboard.html", output_dir);
    println!();
    println!("🔗 Useful Links:");
    println!("  Holesky Explorer: https://holesky.etherscan.io/");
    println!("  Faucet: https://faucet.quicknode.com/ethereum/holesky");
    println!();

    Ok(())
}

/// Save deployment artifacts to files
async fn save_deployment_artifacts(
    deployment_result: &DeploymentResult,
    output_dir: &str,
) -> Result<()> {
    println!("💾 Saving deployment artifacts...");

    // Save full deployment result
    let deployment_file = format!("{}/deployment_result.json", output_dir);
    let deployment_json = serde_json::to_string_pretty(deployment_result)?;
    fs::write(&deployment_file, deployment_json)?;
    println!("✅ Deployment result saved to {}", deployment_file);

    // Save individual contract addresses
    let contracts_file = format!("{}/contracts.json", output_dir);
    let contracts_json = serde_json::to_string_pretty(&deployment_result.contracts)?;
    fs::write(&contracts_file, contracts_json)?;
    println!("✅ Contract addresses saved to {}", contracts_file);

    // Save transaction hashes
    let tx_file = format!("{}/transactions.txt", output_dir);
    let tx_content = deployment_result
        .transaction_hashes
        .iter()
        .enumerate()
        .map(|(i, hash)| format!("Transaction {}: {}", i + 1, hash))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&tx_file, tx_content)?;
    println!("✅ Transaction hashes saved to {}", tx_file);

    Ok(())
}

/// Create solver configuration file
async fn create_solver_config(
    deployment_result: &DeploymentResult,
    output_dir: &str,
) -> Result<()> {
    println!("⚙️ Creating solver configuration...");

    let solver_config = serde_json::json!({
        "solver": {
            "address": deployment_result.config.deployer_address,
            "private_key": deployment_result.config.private_key,
            "supported_chains": [17000],
            "min_profit_bps": 50,
            "max_exposure": "100000000000000000000",
            "reputation_threshold": 5000
        },
        "network": {
            "chain_id": deployment_result.config.chain_id,
            "rpc_url": deployment_result.config.rpc_url,
            "explorer_url": "https://holesky.etherscan.io"
        },
        "contracts": {
            "intents": deployment_result.contracts.intents_contract,
            "orbital_amm": deployment_result.contracts.orbital_amm_contract,
            "mock_usdc": deployment_result.contracts.mock_usdc_contract
        },
        "deployment": {
            "block": deployment_result.contracts.deployment_block,
            "timestamp": deployment_result.contracts.deployment_timestamp,
            "total_gas_used": deployment_result.total_gas_used,
            "total_cost_wei": deployment_result.total_cost
        }
    });

    let config_file = format!("{}/solver_config.json", output_dir);
    let config_json = serde_json::to_string_pretty(&solver_config)?;
    fs::write(&config_file, config_json)?;
    println!("✅ Solver configuration saved to {}", config_file);

    Ok(())
}

/// Create monitoring dashboard
async fn create_monitoring_dashboard(
    deployment_result: &DeploymentResult,
    output_dir: &str,
) -> Result<()> {
    println!("📊 Creating monitoring dashboard...");

    let dashboard_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust Intents - Holesky Deployment Dashboard</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 10px; text-align: center; margin-bottom: 20px; }}
        .deployment-info {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin-bottom: 20px; }}
        .info-card {{ background: white; padding: 20px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
        .contract-address {{ font-family: monospace; background: #f8f9fa; padding: 5px; border-radius: 3px; word-break: break-all; }}
        .status-badge {{ display: inline-block; padding: 4px 8px; border-radius: 4px; font-size: 0.8em; font-weight: bold; }}
        .status-success {{ background: #d4edda; color: #155724; }}
        .status-info {{ background: #d1ecf1; color: #0c5460; }}
        .btn {{ background: #667eea; color: white; padding: 10px 20px; border: none; border-radius: 5px; cursor: pointer; margin: 5px; text-decoration: none; display: inline-block; }}
        .btn:hover {{ background: #5a6fd8; }}
        .tx-list {{ max-height: 200px; overflow-y: auto; }}
        .tx-item {{ padding: 8px; border-bottom: 1px solid #eee; font-family: monospace; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🚀 Rust Intents System</h1>
            <p>Holesky Testnet Deployment Dashboard</p>
            <div class="status-badge status-success">✅ DEPLOYED</div>
        </div>
        
        <div class="deployment-info">
            <div class="info-card">
                <h3>📦 Deployment Info</h3>
                <p><strong>Network:</strong> Holesky Testnet</p>
                <p><strong>Chain ID:</strong> {}</p>
                <p><strong>Deployer:</strong> <span class="contract-address">{}</span></p>
                <p><strong>Block:</strong> {}</p>
                <p><strong>Gas Used:</strong> {}</p>
                <p><strong>Total Cost:</strong> {} ETH</p>
            </div>
            
            <div class="info-card">
                <h3>📋 Smart Contracts</h3>
                <p><strong>Intents Contract:</strong></p>
                <div class="contract-address">{}</div>
                <p><strong>Orbital AMM:</strong></p>
                <div class="contract-address">{}</div>
                <p><strong>Mock USDC:</strong></p>
                <div class="contract-address">{}</div>
            </div>
            
            <div class="info-card">
                <h3>🔗 Transaction Hashes</h3>
                <div class="tx-list">
                    {}
                </div>
            </div>
        </div>
        
        <div class="info-card">
            <h3>🚀 Quick Actions</h3>
            <a href="https://holesky.etherscan.io/address/{}" target="_blank" class="btn">🔍 View Intents Contract</a>
            <a href="https://holesky.etherscan.io/address/{}" target="_blank" class="btn">🔍 View AMM Contract</a>
            <a href="https://holesky.etherscan.io/address/{}" target="_blank" class="btn">🔍 View USDC Contract</a>
            <a href="solver_config.json" class="btn">⚙️ Download Config</a>
            <button class="btn" onclick="copyConfig()">📋 Copy Config</button>
        </div>
        
        <div class="info-card">
            <h3>📚 Next Steps</h3>
            <ol>
                <li>Run the solver: <code>cargo run --bin solver -- --config solver_config.json</code></li>
                <li>Start the demo: <code>./scripts/demo_holesky.sh</code></li>
                <li>Monitor transactions on <a href="https://holesky.etherscan.io" target="_blank">Holesky Explorer</a></li>
                <li>Get test ETH from <a href="https://faucet.quicknode.com/ethereum/holesky" target="_blank">Holesky Faucet</a></li>
            </ol>
        </div>
    </div>
    
    <script>
        function copyConfig() {{
            fetch('solver_config.json')
                .then(response => response.text())
                .then(text => {{
                    navigator.clipboard.writeText(text);
                    alert('Solver configuration copied to clipboard!');
                }});
        }}
    </script>
</body>
</html>"#,
        deployment_result.config.chain_id,
        deployment_result.config.deployer_address,
        deployment_result.contracts.deployment_block,
        deployment_result.total_gas_used,
        ethers::utils::format_ether(deployment_result.total_cost),
        deployment_result.contracts.intents_contract,
        deployment_result.contracts.orbital_amm_contract,
        deployment_result.contracts.mock_usdc_contract,
        deployment_result
            .transaction_hashes
            .iter()
            .enumerate()
            .map(|(i, hash)| format!(
                r#"<div class="tx-item"><strong>{}:</strong> <a href="https://holesky.etherscan.io/tx/{}" target="_blank">{}</a></div>"#,
                i + 1,
                hash,
                hash
            ))
            .collect::<Vec<_>>()
            .join(""),
        deployment_result.contracts.intents_contract,
        deployment_result.contracts.orbital_amm_contract,
        deployment_result.contracts.mock_usdc_contract,
    );

    let dashboard_file = format!("{}/dashboard.html", output_dir);
    fs::write(&dashboard_file, dashboard_html)?;
    println!("✅ Dashboard created at {}", dashboard_file);

    Ok(())
}

/// Verify deployment
async fn verify_deployment(deployment_result: &DeploymentResult) -> Result<()> {
    println!("🔍 Verifying deployment...");

    // Create new client for verification
    use ethers::{middleware::SignerMiddleware, prelude::*};

    let provider = Provider::<ethers::providers::Http>::try_from(&deployment_result.config.rpc_url)?;
    let wallet: LocalWallet = deployment_result.config.private_key.parse::<LocalWallet>()?
        .with_chain_id(deployment_result.config.chain_id);
    let client = SignerMiddleware::new(provider, wallet);

    // Verify each contract has code
    for (name, address) in [
        ("Intents", deployment_result.contracts.intents_contract),
        ("Orbital AMM", deployment_result.contracts.orbital_amm_contract),
        ("Mock USDC", deployment_result.contracts.mock_usdc_contract),
    ] {
        let code = client.get_code(address, None).await?;
        if code.is_empty() {
            return Err(eyre::eyre!("{} contract has no code at {}", name, address));
        }
        println!("✅ {} contract verified", name);
    }

    // Verify transaction receipts
    for (i, tx_hash) in deployment_result.transaction_hashes.iter().enumerate() {
        if let Ok(Some(receipt)) = client.get_transaction_receipt(*tx_hash).await {
            if receipt.status == Some(1.into()) {
                println!("✅ Transaction {} verified", i + 1);
            } else {
                return Err(eyre::eyre!("Transaction {} failed", i + 1));
            }
        } else {
            return Err(eyre::eyre!("Transaction {} not found", i + 1));
        }
    }

    println!("✅ All verifications passed!");
    Ok(())
}