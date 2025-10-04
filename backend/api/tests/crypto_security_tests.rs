use intents_api::crypto::*;
use intents_api::error::ApiError;
use ethers::{
    types::{Address, U256},
    signers::{LocalWallet, Signer},
};
use std::str::FromStr;

#[tokio::test]
async fn test_comprehensive_signature_security() {
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    let address = wallet.address();
    
    // Test various message types
    let test_cases = vec![
        b"Simple message".to_vec(),
        b"Message with special chars: \x00\xff\n\t".to_vec(),
        vec![0u8; 100], // Binary data
        "Unicode message: 🦀 Rust crypto".as_bytes().to_vec(),
    ];
    
    for message in test_cases {
        let signature = wallet.sign_message(&message).await.unwrap();
        let sig_str = format!("{}", signature);
        
        // Should verify successfully
        let result = verify_signature(&message, &sig_str, address);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Should fail with wrong address
        let wrong_address = Address::random();
        let result = verify_signature(&message, &sig_str, wrong_address);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

#[test]
fn test_signature_format_attacks() {
    let address = Address::random();
    let message = b"test message";
    
    // Test various malformed signatures
    let malformed_signatures = vec![
        "", // Empty
        "0x", // Just prefix
        "0x123", // Too short
        "invalid_hex", // Not hex
        "0x" + &"g".repeat(130), // Invalid hex chars
        "0x" + &"0".repeat(128), // Wrong length (64 bytes)
        "0x" + &"f".repeat(132), // Wrong length (66 bytes)
    ];
    
    for sig in malformed_signatures {
        let result = verify_signature(message, sig, address);
        assert!(result.is_err(), "Should fail for signature: {}", sig);
    }
}

#[test]
fn test_timing_attack_resistance() {
    let address1 = Address::from_str("0x742d35cc6634c0532925a3b8d238e78ce6635aa6").unwrap();
    let address2 = Address::from_str("0x742d35cc6634c0532925a3b8d238e78ce6635aa7").unwrap(); // Different by 1 bit
    
    let message = b"timing test message";
    let fake_sig = "0x" + &"a".repeat(130);
    
    // Both should fail, but timing should be consistent
    let start1 = std::time::Instant::now();
    let _ = verify_signature(message, fake_sig, address1);
    let duration1 = start1.elapsed();
    
    let start2 = std::time::Instant::now();
    let _ = verify_signature(message, fake_sig, address2);
    let duration2 = start2.elapsed();
    
    // Timing difference should be minimal (< 10ms difference)
    let diff = if duration1 > duration2 {
        duration1 - duration2
    } else {
        duration2 - duration1
    };
    
    assert!(diff.as_millis() < 10, "Potential timing attack vulnerability");
}

#[test]
fn test_nonce_security_properties() {
    let nonces: Vec<U256> = (0..1000).map(|_| generate_secure_nonce()).collect();
    
    // All nonces should be unique
    for i in 0..nonces.len() {
        for j in (i + 1)..nonces.len() {
            assert_ne!(nonces[i], nonces[j], "Nonces should be unique");
        }
    }
    
    // No nonce should be zero
    for nonce in &nonces {
        assert!(!nonce.is_zero(), "Nonces should never be zero");
    }
    
    // Nonces should have good entropy (rough check)
    let mut bit_counts = [0; 256];
    for nonce in &nonces {
        let bytes = nonce.as_le_bytes();
        for (byte_idx, &byte) in bytes.iter().enumerate() {
            for bit_idx in 0..8 {
                if (byte >> bit_idx) & 1 == 1 {
                    bit_counts[byte_idx * 8 + bit_idx] += 1;
                }
            }
        }
    }
    
    // Each bit should be set roughly 50% of the time (with some tolerance)
    for count in bit_counts.iter() {
        let ratio = *count as f64 / nonces.len() as f64;
        assert!(ratio > 0.4 && ratio < 0.6, "Poor entropy in nonce generation");
    }
}

#[test]
fn test_replay_attack_prevention() {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Test various scenarios
    let test_cases = vec![
        (current_time, 300, true),          // Current time, should pass
        (current_time - 100, 300, true),   // 100s ago, should pass
        (current_time + 100, 300, true),   // 100s future, should pass
        (current_time - 400, 300, false),  // 400s ago, should fail
        (current_time + 400, 300, false),  // 400s future, should fail
        (0, 300, false),                   // Unix epoch, should fail
        (u64::MAX, 300, false),           // Far future, should fail
    ];
    
    for (timestamp, tolerance, should_pass) in test_cases {
        let result = verify_message_freshness(timestamp, tolerance);
        if should_pass {
            assert!(result.is_ok(), "Should pass for timestamp: {}", timestamp);
        } else {
            assert!(result.is_err(), "Should fail for timestamp: {}", timestamp);
        }
    }
}

#[test]
fn test_dos_protection() {
    let limiter = SignatureRateLimiter::new(5, 60); // 5 attempts per minute
    
    // Test normal usage
    for i in 0..5 {
        let result = limiter.check_rate_limit("user1");
        assert!(result.is_ok(), "Attempt {} should succeed", i + 1);
    }
    
    // 6th attempt should fail
    let result = limiter.check_rate_limit("user1");
    assert!(result.is_err(), "6th attempt should fail");
    
    // Different user should not be affected
    let result = limiter.check_rate_limit("user2");
    assert!(result.is_ok(), "Different user should not be rate limited");
}

#[tokio::test]
async fn test_secure_solver_registration() {
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    let address = wallet.address();
    let bond_amount = U256::from(1000000000000000000u64); // 1 ETH
    let supported_chains = vec![1, 137];
    let fee_rate = 50.0;
    let nonce = generate_secure_nonce();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let chain_id = 1;
    
    // Create secure message
    let message = create_secure_solver_registration_message(
        address,
        &bond_amount,
        &supported_chains,
        fee_rate,
        nonce,
        timestamp,
        chain_id,
    );
    
    // Sign and verify
    let signature = wallet.sign_message(&message).await.unwrap();
    let sig_str = format!("{}", signature);
    
    let is_valid = verify_signature(&message, &sig_str, address).unwrap();
    assert!(is_valid);
    
    // Verify message contains all required fields
    let message_str = String::from_utf8(message).unwrap();
    assert!(message_str.contains("v2"));
    assert!(message_str.contains(&format!("{:#x}", address)));
    assert!(message_str.contains(&nonce.to_string()));
    assert!(message_str.contains(&timestamp.to_string()));
    assert!(message_str.contains("5 minutes"));
}

#[test]
fn test_message_size_limits() {
    let address = Address::random();
    let sig = "0x" + &"a".repeat(130);
    
    // Empty message should fail
    let result = validate_signature_inputs(b"", sig, address);
    assert!(result.is_err());
    
    // Maximum allowed size should pass
    let max_message = vec![0u8; 10_000];
    let result = validate_signature_inputs(&max_message, sig, address);
    assert!(result.is_ok());
    
    // Oversized message should fail
    let oversized_message = vec![0u8; 10_001];
    let result = validate_signature_inputs(&oversized_message, sig, address);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_signature_verification_consistency() {
    // Test that both verification methods always agree
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    let address = wallet.address();
    
    let test_messages = vec![
        b"consistency test 1".to_vec(),
        b"consistency test 2 with numbers 12345".to_vec(),
        vec![0u8; 1000], // Large message
        b"\x00\x01\x02\x03\xff\xfe\xfd".to_vec(), // Binary data
    ];
    
    for message in test_messages {
        let signature = wallet.sign_message(&message).await.unwrap();
        let sig_str = format!("{}", signature);
        
        // Test with correct address
        let result = verify_signature(&message, &sig_str, address);
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Test with wrong address  
        let wrong_address = Address::random();
        let result = verify_signature(&message, &sig_str, wrong_address);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}

#[test]
fn test_error_handling_robustness() {
    let address = Address::random();
    let message = b"test";
    
    // Test various error conditions
    let error_cases = vec![
        ("", "Empty signature"),
        ("not_hex", "Invalid hex"),
        ("0x123", "Wrong length"),
        ("0x" + &"z".repeat(130), "Invalid hex chars"),
    ];
    
    for (sig, description) in error_cases {
        let result = verify_signature(message, sig, address);
        assert!(result.is_err(), "Should fail for: {}", description);
        
        // Verify error message is helpful
        if let Err(ApiError::Validation(msg)) = result {
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }
}