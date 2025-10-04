use ethers::{
    prelude::*,
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use serde_json::json;

// Create a simple function for testing message creation
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

fn verify_signature(
    message: &[u8],
    signature: &str,
    expected_signer: Address,
) -> Result<bool, Box<dyn std::error::Error>> {
    use ethers::utils::hash_message;
    use std::str::FromStr;

    let sig = Signature::from_str(signature)?;
    let message_hash = hash_message(message);
    
    match sig.recover(message_hash) {
        Ok(recovered_address) => Ok(recovered_address == expected_signer),
        Err(e) => Err(e.into()),
    }
}

#[tokio::test]
async fn test_solver_registration_signature() {
    // Create a test wallet
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    let solver_address = wallet.address();
    
    // Prepare registration data
    let bond_amount = U256::from(1000000000000000000u64); // 1 ETH
    let supported_chains = vec![1, 137, 42161]; // Ethereum, Polygon, Arbitrum
    let fee_rate = 30.0; // 0.3%
    
    // Create the message to sign
    let message = create_solver_registration_message(
        solver_address,
        &bond_amount,
        &supported_chains,
        fee_rate,
    );
    
    // Sign the message
    let signature = wallet.sign_message(&message).await.unwrap();
    let sig_str = format!("{}", signature);
    
    // Verify the signature works
    let is_valid = verify_signature(
        &message,
        &sig_str,
        solver_address,
    ).unwrap();
    
    assert!(is_valid);
    
    // Create registration request
    let registration_request = json!({
        "solver_address": format!("{:#x}", solver_address),
        "bond_amount": bond_amount.to_string(),
        "supported_chains": supported_chains,
        "fee_rate": fee_rate,
        "contact_info": "test@example.com",
        "signature": sig_str
    });
    
    println!("Test registration request: {}", serde_json::to_string_pretty(&registration_request).unwrap());
}

#[tokio::test]
async fn test_invalid_signature() {
    // Create two different wallets
    let wallet1 = LocalWallet::new(&mut rand::thread_rng());
    let wallet2 = LocalWallet::new(&mut rand::thread_rng());
    
    let solver_address = wallet1.address();
    let bond_amount = U256::from(1000000000000000000u64);
    let supported_chains = vec![1];
    let fee_rate = 30.0;
    
    // Create message for wallet1
    let message = create_solver_registration_message(
        solver_address,
        &bond_amount,
        &supported_chains,
        fee_rate,
    );
    
    // Sign with wallet2 (wrong wallet)
    let signature = wallet2.sign_message(&message).await.unwrap();
    let sig_str = format!("{}", signature);
    
    // Verify should fail
    let is_valid = verify_signature(
        &message,
        &sig_str,
        solver_address,
    ).unwrap();
    
    assert!(!is_valid);
}

#[test]
fn test_registration_message_format() {
    let solver_address = Address::random();
    let bond_amount = U256::from(2500000000000000000u64); // 2.5 ETH
    let supported_chains = vec![1, 10, 100]; // Ethereum, Optimism, Gnosis
    let fee_rate = 50.0; // 0.5%
    
    let message = create_solver_registration_message(
        solver_address,
        &bond_amount,
        &supported_chains,
        fee_rate,
    );
    
    let message_str = String::from_utf8(message).unwrap();
    
    // Verify message contains expected content
    assert!(message_str.contains("Orbital Intents Solver Registration"));
    assert!(message_str.contains(&format!("{:#x}", solver_address)));
    assert!(message_str.contains("2500000000000000000 wei"));
    assert!(message_str.contains("1,10,100"));
    assert!(message_str.contains("50 bps"));
    assert!(message_str.contains("I am the owner of the solver address"));
    assert!(message_str.contains("my bond may be slashed"));
}