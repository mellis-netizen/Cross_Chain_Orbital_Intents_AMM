use ethers::{
    prelude::*,
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use serde_json::json;
use std::str::FromStr;

/// Example of how to create a solver registration with signature verification
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Orbital Intents Solver Registration Example ===\n");

    // Example 1: Create a test wallet (in production, use your actual private key)
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    let solver_address = wallet.address();
    
    println!("Generated test wallet:");
    println!("Address: {:#x}", solver_address);
    println!("Private Key: {:#x}", wallet.signer().to_bytes());
    println!();

    // Example 2: Define registration parameters
    let bond_amount = U256::from_dec_str("2500000000000000000")?; // 2.5 ETH
    let supported_chains = vec![1, 137, 42161]; // Ethereum, Polygon, Arbitrum
    let fee_rate = 30.0; // 0.3% (30 basis points)
    let contact_info = Some("solver@example.com".to_string());

    println!("Registration parameters:");
    println!("Bond Amount: {} ETH", ethers::utils::format_ether(bond_amount));
    println!("Supported Chains: {:?}", supported_chains);
    println!("Fee Rate: {} bps ({}%)", fee_rate, fee_rate / 100.0);
    println!("Contact Info: {:?}", contact_info);
    println!();

    // Example 3: Create the message to sign
    let message = create_solver_registration_message(
        solver_address,
        &bond_amount,
        &supported_chains,
        fee_rate,
    );
    
    let message_str = String::from_utf8(message.clone())?;
    println!("Message to sign:");
    println!("{}", message_str);
    println!();

    // Example 4: Sign the message
    let signature = wallet.sign_message(&message).await?;
    let signature_str = format!("{}", signature);
    
    println!("Generated signature:");
    println!("{}", signature_str);
    println!();

    // Example 5: Verify the signature (this is what the server does)
    let is_valid = verify_signature(&message, &signature_str, solver_address)?;
    println!("Signature verification: {}", if is_valid { "✅ VALID" } else { "❌ INVALID" });
    println!();

    // Example 6: Create the registration request JSON
    let registration_request = json!({
        "solver_address": format!("{:#x}", solver_address),
        "bond_amount": bond_amount.to_string(),
        "supported_chains": supported_chains,
        "fee_rate": fee_rate,
        "contact_info": contact_info,
        "signature": signature_str
    });

    println!("Complete registration request:");
    println!("{}", serde_json::to_string_pretty(&registration_request)?);
    println!();

    // Example 7: Test with wrong signature (should fail)
    let wrong_wallet = LocalWallet::new(&mut rand::thread_rng());
    let wrong_signature = wrong_wallet.sign_message(&message).await?;
    let wrong_signature_str = format!("{}", wrong_signature);
    
    let is_wrong_valid = verify_signature(&message, &wrong_signature_str, solver_address)?;
    println!("Wrong signature verification: {}", if is_wrong_valid { "❌ UNEXPECTED VALID" } else { "✅ CORRECTLY INVALID" });
    println!();

    // Example 8: Show curl command for API call
    println!("Example curl command to register:");
    println!("curl -X POST http://localhost:3000/api/v1/solver/register \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{}'", serde_json::to_string(&registration_request)?);
    println!();

    println!("✅ Example completed successfully!");
    Ok(())
}

/// Create the standardized message for solver registration
fn create_solver_registration_message(
    solver_address: Address,
    bond_amount: &U256,
    supported_chains: &[u64],
    fee_rate: f64,
) -> Vec<u8> {
    let chains_str = supported_chains
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(",");
    
    let message = format!(
        "Orbital Intents Solver Registration\n\
         Solver Address: {:#x}\n\
         Bond Amount: {} wei\n\
         Supported Chains: [{}]\n\
         Fee Rate: {} bps\n\
         \n\
         By signing this message, I confirm that:\n\
         - I am the owner of the solver address\n\
         - I agree to the solver terms and conditions\n\
         - I understand that my bond may be slashed for misbehavior",
        solver_address,
        bond_amount,
        chains_str,
        fee_rate
    );
    
    message.as_bytes().to_vec()
}

/// Verify an Ethereum signature for a given message and expected signer
fn verify_signature(
    message: &[u8],
    signature: &str,
    expected_signer: Address,
) -> Result<bool, Box<dyn std::error::Error>> {
    use ethers::utils::hash_message;

    // Parse signature from hex string
    let sig = Signature::from_str(signature)?;
    
    // Hash the message using EIP-191 personal sign format
    let message_hash = hash_message(message);
    
    // Recover the signer address from the signature
    match sig.recover(message_hash) {
        Ok(recovered_address) => Ok(recovered_address == expected_signer),
        Err(e) => Err(e.into()),
    }
}