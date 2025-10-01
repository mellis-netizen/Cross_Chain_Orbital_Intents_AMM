use ethers::types::{Address, U256, H256, Bytes, Signature};
use ethers::core::utils::hash_message;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub user: Address,
    pub source_chain_id: u64,
    pub dest_chain_id: u64,
    pub source_token: Address,
    pub dest_token: Address,
    pub source_amount: U256,
    pub min_dest_amount: U256,
    pub deadline: u64,
    pub nonce: U256,
    pub data: Option<Bytes>,
    pub signature: Bytes,
}

impl Default for Intent {
    fn default() -> Self {
        Self {
            user: Address::zero(),
            source_chain_id: 1,
            dest_chain_id: 137,
            source_token: Address::zero(),
            dest_token: Address::zero(),
            source_amount: U256::zero(),
            min_dest_amount: U256::zero(),
            deadline: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() + 3600, // 1 hour from now
            nonce: U256::zero(),
            data: None,
            signature: Bytes::default(),
        }
    }
}

impl Intent {
    /// Check if the intent has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.deadline
    }

    /// Generate intent ID from hash of intent data
    pub fn id(&self) -> H256 {
        use ethers::core::utils::keccak256;
        
        let mut data = Vec::new();
        data.extend_from_slice(self.user.as_bytes());
        data.extend_from_slice(&self.source_chain_id.to_le_bytes());
        data.extend_from_slice(&self.dest_chain_id.to_le_bytes());
        data.extend_from_slice(self.source_token.as_bytes());
        data.extend_from_slice(self.dest_token.as_bytes());
        data.extend_from_slice(&self.source_amount.to_le_bytes());
        data.extend_from_slice(&self.min_dest_amount.to_le_bytes());
        data.extend_from_slice(&self.deadline.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        
        H256::from(keccak256(data))
    }

    /// Verify the intent signature
    pub fn verify_signature(&self) -> bool {
        // TODO: Implement EIP-712 signature verification
        // For now, return true if signature is present
        !self.signature.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentStatus {
    Pending,
    Matched,
    Executing,
    Executed,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentExecution {
    pub intent_id: H256,
    pub solver: Address,
    pub dest_amount: U256,
    pub execution_proof: Vec<u8>,
    pub gas_used: U256,
    pub execution_time: u64,
    pub source_tx_hash: H256,
    pub dest_tx_hash: H256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Matched,
    Executing,
    Completed,
    Failed,
}

impl Intent {
    pub fn compute_id(&self) -> H256 {
        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::Address(self.user),
            ethers::abi::Token::Uint(self.source_chain_id.into()),
            ethers::abi::Token::Uint(self.dest_chain_id.into()),
            ethers::abi::Token::Address(self.source_token),
            ethers::abi::Token::Address(self.dest_token),
            ethers::abi::Token::Uint(self.source_amount),
            ethers::abi::Token::Uint(self.min_dest_amount),
            ethers::abi::Token::Uint(self.deadline.into()),
            ethers::abi::Token::Uint(self.nonce),
        ]);
        
        ethers::utils::keccak256(encoded).into()
    }
    
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.deadline < now
    }
    
    pub fn verify_signature(&self) -> bool {
        // EIP-712 Domain Separator
        let domain_separator = self.compute_domain_separator();

        // Compute struct hash (intent ID)
        let struct_hash = self.compute_id();

        // Create EIP-712 typed data hash
        // \x19\x01 is the EIP-712 prefix
        let typed_data_hash = {
            let mut bytes = Vec::with_capacity(66);
            bytes.extend_from_slice(&[0x19, 0x01]);
            bytes.extend_from_slice(domain_separator.as_bytes());
            bytes.extend_from_slice(struct_hash.as_bytes());
            H256::from_slice(&ethers::utils::keccak256(&bytes))
        };

        // Parse signature from bytes
        let signature = match Signature::try_from(self.signature.as_ref()) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        // Recover signer address from signature
        match signature.recover(typed_data_hash) {
            Ok(recovered_address) => {
                // Verify recovered address matches the intent user
                recovered_address == self.user
            },
            Err(_) => false,
        }
    }

    fn compute_domain_separator(&self) -> H256 {
        // EIP-712 Domain Separator
        // keccak256(abi.encode(
        //     keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
        //     keccak256("OrbitalIntents"),
        //     keccak256("1"),
        //     chainId,
        //     verifyingContract
        // ))

        let type_hash = ethers::utils::keccak256(
            "EIP712Domain(string name,string version,uint256 chainId)"
        );
        let name_hash = ethers::utils::keccak256("OrbitalIntents");
        let version_hash = ethers::utils::keccak256("1");

        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(type_hash.to_vec()),
            ethers::abi::Token::FixedBytes(name_hash.to_vec()),
            ethers::abi::Token::FixedBytes(version_hash.to_vec()),
            ethers::abi::Token::Uint(self.source_chain_id.into()),
        ]);

        H256::from_slice(&ethers::utils::keccak256(encoded))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    pub message_type: MessageType,
    pub source_chain: u64,
    pub dest_chain: u64,
    pub nonce: u64,
    pub payload: Bytes,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    IntentCreated,
    IntentMatched,
    IntentExecuted,
    IntentCancelled,
    LiquidityUpdate,
    PriceUpdate,
}