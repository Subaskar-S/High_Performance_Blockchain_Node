use anyhow::{Result, anyhow};
use std::collections::HashSet;
use crate::types::{Block, Transaction, BlockHeader, Hash, Address, BlockHeight};
use crate::storage::Storage;

/// Block validation errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid block hash")]
    InvalidBlockHash,
    #[error("Invalid previous hash")]
    InvalidPreviousHash,
    #[error("Invalid merkle root")]
    InvalidMerkleRoot,
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),
    #[error("Invalid block height: expected {expected}, got {actual}")]
    InvalidBlockHeight { expected: BlockHeight, actual: BlockHeight },
    #[error("Transaction validation failed: {0}")]
    TransactionValidation(String),
    #[error("Duplicate transaction: {0}")]
    DuplicateTransaction(String),
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid nonce: expected {expected}, got {actual}")]
    InvalidNonce { expected: u64, actual: u64 },
}

/// Comprehensive blockchain validator
pub struct Validator {
    storage: Storage,
}

impl Validator {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Validate a complete block
    pub fn validate_block(&self, block: &Block) -> Result<()> {
        // 1. Validate block structure
        self.validate_block_structure(block)?;
        
        // 2. Validate block header
        self.validate_block_header(&block.header)?;
        
        // 3. Validate all transactions
        self.validate_block_transactions(block)?;
        
        // 4. Validate merkle root
        self.validate_merkle_root(block)?;
        
        // 5. Validate against previous block
        self.validate_block_chain(block)?;
        
        Ok(())
    }

    /// Validate block structure and basic integrity
    fn validate_block_structure(&self, block: &Block) -> Result<()> {
        // Check if block has transactions
        if block.transactions.is_empty() {
            return Err(anyhow!("Block must contain at least one transaction"));
        }

        // Check block size limits (simplified)
        let serialized = bincode::serialize(block)
            .map_err(|e| anyhow!("Failed to serialize block: {}", e))?;
        
        if serialized.len() > 1024 * 1024 * 10 { // 10MB limit
            return Err(anyhow!("Block size exceeds maximum limit"));
        }

        // Verify block hash integrity
        if !block.verify() {
            return Err(ValidationError::InvalidBlockHash.into());
        }

        Ok(())
    }

    /// Validate block header
    fn validate_block_header(&self, header: &BlockHeader) -> Result<()> {
        // Validate timestamp (not too far in future)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        if header.timestamp > current_time + 300_000 { // 5 minutes tolerance
            return Err(ValidationError::InvalidTimestamp(
                "Block timestamp too far in future".to_string()
            ).into());
        }

        // Validate height sequence
        if let Some(latest_height) = self.storage.get_latest_height()? {
            let expected_height = latest_height + 1;
            if header.height != expected_height {
                return Err(ValidationError::InvalidBlockHeight {
                    expected: expected_height,
                    actual: header.height,
                }.into());
            }
        } else if header.height != 0 {
            // Genesis block should have height 0
            return Err(ValidationError::InvalidBlockHeight {
                expected: 0,
                actual: header.height,
            }.into());
        }

        Ok(())
    }

    /// Validate all transactions in the block
    fn validate_block_transactions(&self, block: &Block) -> Result<()> {
        let mut seen_tx_hashes = HashSet::new();
        let mut account_nonces: std::collections::HashMap<Address, u64> = std::collections::HashMap::new();

        for tx in &block.transactions {
            // Check for duplicate transactions within block
            let tx_hash = tx.hash();
            if seen_tx_hashes.contains(&tx_hash) {
                return Err(ValidationError::DuplicateTransaction(
                    format!("{:?}", tx.id)
                ).into());
            }
            seen_tx_hashes.insert(tx_hash);

            // Validate individual transaction
            self.validate_transaction(tx)?;

            // Validate nonce sequence within block
            let current_nonce = account_nonces.get(&tx.from).copied()
                .unwrap_or_else(|| {
                    // Get nonce from storage
                    self.storage.state().get_nonce(&tx.from).unwrap_or(0)
                });

            if tx.nonce != current_nonce + 1 {
                return Err(ValidationError::InvalidNonce {
                    expected: current_nonce + 1,
                    actual: tx.nonce,
                }.into());
            }

            account_nonces.insert(tx.from, tx.nonce);

            // Validate balance (simplified - doesn't account for all state changes)
            let balance = self.storage.state().get_balance(&tx.from)?;
            if balance < tx.amount + tx.fee {
                return Err(ValidationError::InsufficientBalance.into());
            }
        }

        Ok(())
    }

    /// Validate individual transaction
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Basic transaction validation
        if !tx.verify() {
            return Err(ValidationError::InvalidSignature.into());
        }

        // Check transaction fields
        if tx.amount == 0 && tx.data.is_empty() {
            return Err(anyhow!("Transaction must transfer value or contain data"));
        }

        if tx.fee == 0 {
            return Err(anyhow!("Transaction must include fee"));
        }

        if tx.from == tx.to && tx.data.is_empty() {
            return Err(anyhow!("Self-transfer without data is not allowed"));
        }

        // Validate timestamp (not too old or too far in future)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if tx.timestamp > current_time + 60_000 { // 1 minute tolerance
            return Err(anyhow!("Transaction timestamp too far in future"));
        }

        if current_time - tx.timestamp > 3600_000 { // 1 hour max age
            return Err(anyhow!("Transaction too old"));
        }

        Ok(())
    }

    /// Validate merkle root
    fn validate_merkle_root(&self, block: &Block) -> Result<()> {
        let calculated_root = block.calculate_merkle_root();
        if calculated_root != block.header.merkle_root {
            return Err(ValidationError::InvalidMerkleRoot.into());
        }
        Ok(())
    }

    /// Validate block against blockchain history
    fn validate_block_chain(&self, block: &Block) -> Result<()> {
        // For genesis block, skip chain validation
        if block.header.height == 0 {
            return Ok(());
        }

        // Get previous block
        let prev_height = block.header.height - 1;
        let prev_block = self.storage.blocks().get_block(prev_height)?
            .ok_or_else(|| anyhow!("Previous block not found"))?;

        // Validate previous hash
        let prev_hash = prev_block.hash();
        if prev_hash != block.header.previous_hash {
            return Err(ValidationError::InvalidPreviousHash.into());
        }

        // Validate timestamp ordering
        if block.header.timestamp <= prev_block.header.timestamp {
            return Err(ValidationError::InvalidTimestamp(
                "Block timestamp must be greater than previous block".to_string()
            ).into());
        }

        Ok(())
    }

    /// Validate transaction against current state
    pub fn validate_transaction_against_state(&self, tx: &Transaction) -> Result<()> {
        // Check balance
        let balance = self.storage.state().get_balance(&tx.from)?;
        if balance < tx.amount + tx.fee {
            return Err(ValidationError::InsufficientBalance.into());
        }

        // Check nonce
        let current_nonce = self.storage.state().get_nonce(&tx.from)?;
        if tx.nonce != current_nonce + 1 {
            return Err(ValidationError::InvalidNonce {
                expected: current_nonce + 1,
                actual: tx.nonce,
            }.into());
        }

        Ok(())
    }

    /// Validate a batch of transactions for mempool
    pub fn validate_transaction_batch(&self, transactions: &[Transaction]) -> Result<Vec<bool>> {
        let mut results = Vec::with_capacity(transactions.len());
        
        for tx in transactions {
            let is_valid = self.validate_transaction(tx).is_ok() && 
                          self.validate_transaction_against_state(tx).is_ok();
            results.push(is_valid);
        }
        
        Ok(results)
    }

    /// Quick validation for mempool admission
    pub fn quick_validate_transaction(&self, tx: &Transaction) -> bool {
        // Fast validation without state checks
        tx.verify() && 
        tx.fee > 0 && 
        (tx.amount > 0 || !tx.data.is_empty()) &&
        tx.from != tx.to
    }

    /// Validate consensus signatures on block
    pub fn validate_consensus_signatures(&self, block: &Block, validator_set: &[String]) -> Result<()> {
        if block.signatures.is_empty() {
            return Err(anyhow!("Block must have consensus signatures"));
        }

        // Check if we have enough signatures (2/3 + 1 for BFT)
        let required_signatures = (validator_set.len() * 2 / 3) + 1;
        if block.signatures.len() < required_signatures {
            return Err(anyhow!("Insufficient consensus signatures"));
        }

        // Validate each signature (simplified)
        for sig in &block.signatures {
            if !validator_set.contains(&sig.validator_id) {
                return Err(anyhow!("Invalid validator signature"));
            }
        }

        Ok(())
    }
}

/// Validation statistics
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    pub blocks_validated: u64,
    pub transactions_validated: u64,
    pub validation_errors: u64,
    pub avg_validation_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::types::{BlockHeader, ValidatorSignature};
    use uuid::Uuid;

    fn create_test_storage() -> Storage {
        let temp_dir = TempDir::new().unwrap();
        Storage::new(temp_dir.path()).unwrap()
    }

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            from: [1u8; 20],
            to: [2u8; 20],
            amount: 1000,
            fee: 10,
            nonce: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            signature: [0u8; 64],
            data: vec![],
        }
    }

    fn create_test_block() -> Block {
        let tx = create_test_transaction();
        let mut block = Block {
            header: BlockHeader {
                height: 1,
                previous_hash: [0; 32],
                merkle_root: [0; 32],
                state_root: [0; 32],
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                proposer: "test-node".to_string(),
                round: 1,
                view: 1,
            },
            transactions: vec![tx],
            signatures: vec![],
        };
        
        // Calculate correct merkle root
        block.header.merkle_root = block.calculate_merkle_root();
        block
    }

    #[test]
    fn test_transaction_validation() {
        let storage = create_test_storage();
        let validator = Validator::new(storage);
        
        let tx = create_test_transaction();
        assert!(validator.validate_transaction(&tx).is_ok());
    }

    #[test]
    fn test_block_structure_validation() {
        let storage = create_test_storage();
        let validator = Validator::new(storage);
        
        let block = create_test_block();
        assert!(validator.validate_block_structure(&block).is_ok());
    }
}
