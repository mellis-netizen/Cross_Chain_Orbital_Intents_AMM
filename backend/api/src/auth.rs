use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use ethers::types::Address;
use crate::{error::Result, models::Claims};
use std::str::FromStr;

// JWT token management
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_iat = true;
        
        Self {
            encoding_key,
            decoding_key,
            validation,
        }
    }
    
    pub fn generate_token(
        &self,
        user_address: Address,
        role: &str,
        duration: Duration,
    ) -> Result<String> {
        let now = Utc::now();
        let exp = (now + duration).timestamp() as usize;
        let iat = now.timestamp() as usize;
        
        let claims = Claims {
            sub: format!("{:#x}", user_address),
            exp,
            iat,
            role: role.to_string(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| crate::error::ApiError::Jwt(e))
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| crate::error::ApiError::Jwt(e))?;
        
        Ok(token_data.claims)
    }
}

// Public function for middleware
pub fn validate_jwt(token: &str, secret: &str) -> Result<Claims> {
    let manager = JwtManager::new(secret);
    manager.validate_token(token)
}

// Ethereum signature verification
pub fn verify_ethereum_signature(
    message: &str,
    signature: &str,
    expected_address: Address,
) -> Result<bool> {
    use ethers::core::k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
    use ethers::core::utils::keccak256;
    
    // Parse signature
    let signature_bytes = hex::decode(signature.trim_start_matches("0x"))
        .map_err(|_| crate::error::validation_error("Invalid signature format"))?;
    
    if signature_bytes.len() != 65 {
        return Err(crate::error::validation_error("Invalid signature length"));
    }
    
    // Split signature components
    let recovery_id = RecoveryId::try_from(signature_bytes[64] % 27)
        .map_err(|_| crate::error::validation_error("Invalid recovery ID"))?;
    
    let signature = Signature::try_from(&signature_bytes[..64])
        .map_err(|_| crate::error::validation_error("Invalid signature"))?;
    
    // Create message hash (EIP-191)
    let message_hash = create_eth_message_hash(message);
    
    // Recover public key
    let verifying_key = VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id)
        .map_err(|_| crate::error::validation_error("Failed to recover public key"))?;
    
    // Derive address from public key
    let public_key_bytes = verifying_key.to_encoded_point(false);
    let public_key_hash = keccak256(&public_key_bytes.as_bytes()[1..]);
    let recovered_address = Address::from_slice(&public_key_hash[12..]);
    
    Ok(recovered_address == expected_address)
}

// Create Ethereum signed message hash (EIP-191)
pub fn create_eth_message_hash(message: &str) -> [u8; 32] {
    use ethers::core::utils::keccak256;
    
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let full_message = format!("{}{}", prefix, message);
    keccak256(full_message.as_bytes())
}

// Authentication request structures
#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub address: Address,
    pub signature: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub expires_at: i64,
    pub user_address: Address,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthChallengeRequest {
    pub address: Address,
}

#[derive(Debug, Serialize)]
pub struct AuthChallengeResponse {
    pub message: String,
    pub timestamp: i64,
}

// Challenge generation for secure authentication
pub fn generate_auth_challenge(address: Address) -> AuthChallengeResponse {
    let timestamp = Utc::now().timestamp();
    let message = format!(
        "Sign this message to authenticate with Orbital AMM\n\nAddress: {:#x}\nTimestamp: {}\nNonce: {}",
        address,
        timestamp,
        uuid::Uuid::new_v4()
    );
    
    AuthChallengeResponse {
        message,
        timestamp,
    }
}

// Validate authentication challenge
pub fn validate_auth_challenge(
    auth_request: &AuthRequest,
    max_age_seconds: i64,
) -> Result<()> {
    let now = Utc::now().timestamp();
    
    // Check if timestamp is not too old
    if now - auth_request.timestamp > max_age_seconds {
        return Err(crate::error::validation_error("Authentication challenge expired"));
    }
    
    // Check if timestamp is not in the future (allow 5 minutes skew)
    if auth_request.timestamp - now > 300 {
        return Err(crate::error::validation_error("Authentication challenge timestamp is in the future"));
    }
    
    // Verify signature
    if !verify_ethereum_signature(&auth_request.message, &auth_request.signature, auth_request.address)? {
        return Err(crate::error::validation_error("Invalid signature"));
    }
    
    Ok(())
}

// Role management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    User,
    Solver,
    Admin,
}

impl UserRole {
    pub fn from_address(address: Address) -> Self {
        // In a real implementation, this would check the database
        // or smart contract to determine the user's role
        // For now, we'll default to User
        UserRole::User
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Solver => "solver",
            UserRole::Admin => "admin",
        }
    }
    
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "user" => Ok(UserRole::User),
            "solver" => Ok(UserRole::Solver),
            "admin" => Ok(UserRole::Admin),
            _ => Err(crate::error::validation_error("Invalid role")),
        }
    }
    
    pub fn can_access_endpoint(&self, endpoint: &str, method: &str) -> bool {
        match self {
            UserRole::Admin => true, // Admin can access everything
            UserRole::Solver => {
                // Solvers can access solver-specific endpoints
                endpoint.contains("/solver/") || 
                endpoint.contains("/intents") ||
                method == "GET" // Solvers can read most data
            }
            UserRole::User => {
                // Users can access user-specific endpoints
                !endpoint.contains("/admin/") && !endpoint.contains("/solver/admin")
            }
        }
    }
}

// Permission checking
pub fn check_permission(
    claims: &Claims,
    endpoint: &str,
    method: &str,
) -> Result<()> {
    let role = UserRole::from_str(&claims.role)?;
    
    if !role.can_access_endpoint(endpoint, method) {
        return Err(crate::error::ApiError::Authorization(
            "Insufficient permissions".to_string()
        ));
    }
    
    Ok(())
}

// Extract user address from claims
pub fn extract_user_address(claims: &Claims) -> Result<Address> {
    Address::from_str(&claims.sub)
        .map_err(|_| crate::error::validation_error("Invalid user address in token"))
}

// Session management (using Redis)
use crate::cache::CacheService;

pub struct SessionManager;

impl SessionManager {
    pub async fn create_session(
        cache: &mut CacheService,
        user_address: Address,
        token: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<()> {
        let session_key = format!("session:{:#x}", user_address);
        let session_data = serde_json::json!({
            "token": token,
            "expires_at": expires_at.to_rfc3339(),
            "created_at": Utc::now().to_rfc3339()
        });
        
        let ttl = (expires_at - Utc::now()).to_std()
            .map_err(|_| crate::error::internal_error("Invalid TTL calculation"))?;
        
        cache.set(&session_key, &session_data, Some(ttl)).await?;
        
        Ok(())
    }
    
    pub async fn validate_session(
        cache: &mut CacheService,
        user_address: Address,
        token: &str,
    ) -> Result<bool> {
        let session_key = format!("session:{:#x}", user_address);
        
        if let Some(session_data) = cache.get::<serde_json::Value>(&session_key).await? {
            if let Some(stored_token) = session_data.get("token").and_then(|t| t.as_str()) {
                return Ok(stored_token == token);
            }
        }
        
        Ok(false)
    }
    
    pub async fn revoke_session(
        cache: &mut CacheService,
        user_address: Address,
    ) -> Result<()> {
        let session_key = format!("session:{:#x}", user_address);
        cache.delete(&session_key).await?;
        Ok(())
    }
    
    pub async fn cleanup_expired_sessions(
        cache: &mut CacheService,
    ) -> Result<()> {
        // Redis handles TTL automatically, so this is mostly a no-op
        // But we could implement additional cleanup logic here if needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::signers::{LocalWallet, Signer};
    
    #[tokio::test]
    async fn test_jwt_generation_and_validation() {
        let secret = "test-secret";
        let manager = JwtManager::new(secret);
        let address = Address::from_str("0x742d35Cc6632C6532C8b6Fd95E1d9c37D8B99d1c").unwrap();
        
        let token = manager.generate_token(
            address,
            "user",
            Duration::hours(1),
        ).unwrap();
        
        let claims = manager.validate_token(&token).unwrap();
        assert_eq!(claims.sub, format!("{:#x}", address));
        assert_eq!(claims.role, "user");
    }
    
    #[tokio::test]
    async fn test_ethereum_signature_verification() {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let address = wallet.address();
        let message = "Test message";
        
        let signature = wallet.sign_message(message).await.unwrap();
        let signature_hex = format!("{:#x}", signature);
        
        let is_valid = verify_ethereum_signature(message, &signature_hex, address).unwrap();
        assert!(is_valid);
    }
}