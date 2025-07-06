use anyhow::{Result, anyhow};
use rocksdb::DB;
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::types::{Hash, Address, BlockHeight};

const CF_STATE: &str = "state";

/// Account state information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountState {
    pub balance: u64,
    pub nonce: u64,
    pub code_hash: Option<Hash>,
    pub storage_root: Hash,
}

/// State store for managing account states and world state
pub struct StateStore {
    db: Arc<DB>,
}

impl StateStore {
    pub fn new(db: Arc<DB>) -> Result<Self> {
        Ok(Self { db })
    }

    /// Get account state
    pub fn get_account(&self, address: &Address) -> Result<Option<AccountState>> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        let key = self.account_key(address);
        
        match self.db.get_cf(cf, &key)? {
            Some(data) => {
                let state = bincode::deserialize(&data)
                    .map_err(|e| anyhow!("Failed to deserialize account state: {}", e))?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Set account state
    pub fn set_account(&self, address: &Address, state: &AccountState) -> Result<()> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        let key = self.account_key(address);
        let serialized = bincode::serialize(state)
            .map_err(|e| anyhow!("Failed to serialize account state: {}", e))?;
        
        self.db.put_cf(cf, &key, &serialized)
            .map_err(|e| anyhow!("Failed to store account state: {}", e))?;

        Ok(())
    }

    /// Get account balance
    pub fn get_balance(&self, address: &Address) -> Result<u64> {
        match self.get_account(address)? {
            Some(state) => Ok(state.balance),
            None => Ok(0),
        }
    }

    /// Set account balance
    pub fn set_balance(&self, address: &Address, balance: u64) -> Result<()> {
        let mut state = self.get_account(address)?.unwrap_or_default();
        state.balance = balance;
        self.set_account(address, &state)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &Address) -> Result<u64> {
        match self.get_account(address)? {
            Some(state) => Ok(state.nonce),
            None => Ok(0),
        }
    }

    /// Increment account nonce
    pub fn increment_nonce(&self, address: &Address) -> Result<()> {
        let mut state = self.get_account(address)?.unwrap_or_default();
        state.nonce += 1;
        self.set_account(address, &state)
    }

    /// Transfer balance between accounts
    pub fn transfer(&self, from: &Address, to: &Address, amount: u64) -> Result<()> {
        // Get current balances
        let from_balance = self.get_balance(from)?;
        let to_balance = self.get_balance(to)?;

        // Check sufficient balance
        if from_balance < amount {
            return Err(anyhow!("Insufficient balance"));
        }

        // Update balances
        self.set_balance(from, from_balance - amount)?;
        self.set_balance(to, to_balance + amount)?;

        Ok(())
    }

    /// Get state root hash for a given block height
    pub fn get_state_root(&self, height: BlockHeight) -> Result<Option<Hash>> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        let key = self.state_root_key(height);
        
        match self.db.get_cf(cf, &key)? {
            Some(data) => {
                if data.len() == 32 {
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&data);
                    Ok(Some(hash))
                } else {
                    Err(anyhow!("Invalid state root hash length"))
                }
            }
            None => Ok(None),
        }
    }

    /// Set state root hash for a given block height
    pub fn set_state_root(&self, height: BlockHeight, root: &Hash) -> Result<()> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        let key = self.state_root_key(height);
        
        self.db.put_cf(cf, &key, root)
            .map_err(|e| anyhow!("Failed to store state root: {}", e))?;

        Ok(())
    }

    /// Calculate current state root (simplified Merkle tree)
    pub fn calculate_state_root(&self) -> Result<Hash> {
        use sha2::{Digest, Sha256};
        
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        let mut hasher = Sha256::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, value) = item?;
            
            // Only hash account keys (skip state root keys)
            if key.starts_with(b"acc_") {
                hasher.update(&key);
                hasher.update(&value);
            }
        }
        
        Ok(hasher.finalize().into())
    }

    /// Get all accounts (for debugging/testing)
    pub fn get_all_accounts(&self) -> Result<HashMap<Address, AccountState>> {
        let cf = self.db.cf_handle(CF_STATE)
            .ok_or_else(|| anyhow!("State column family not found"))?;
        
        let mut accounts = HashMap::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, value) = item?;
            
            // Only process account keys
            if key.starts_with(b"acc_") {
                if let Ok(address) = self.parse_account_key(&key) {
                    if let Ok(state) = bincode::deserialize(&value) {
                        accounts.insert(address, state);
                    }
                }
            }
        }
        
        Ok(accounts)
    }

    /// Create snapshot of current state
    pub fn create_snapshot(&self, height: BlockHeight) -> Result<()> {
        let state_root = self.calculate_state_root()?;
        self.set_state_root(height, &state_root)?;
        Ok(())
    }

    /// Create account key
    fn account_key(&self, address: &Address) -> Vec<u8> {
        let mut key = Vec::with_capacity(24);
        key.extend_from_slice(b"acc_");
        key.extend_from_slice(address);
        key
    }

    /// Create state root key
    fn state_root_key(&self, height: BlockHeight) -> Vec<u8> {
        format!("root_{:016}", height).into_bytes()
    }

    /// Parse address from account key
    fn parse_account_key(&self, key: &[u8]) -> Result<Address> {
        if key.len() == 24 && key.starts_with(b"acc_") {
            let mut address = [0u8; 20];
            address.copy_from_slice(&key[4..]);
            Ok(address)
        } else {
            Err(anyhow!("Invalid account key format"))
        }
    }
}

impl Default for AccountState {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            code_hash: None,
            storage_root: [0; 32],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use rocksdb::{DB, Options, ColumnFamilyDescriptor};

    fn create_test_db() -> (TempDir, Arc<DB>) {
        let temp_dir = TempDir::new().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_STATE, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, temp_dir.path(), cfs).unwrap();
        (temp_dir, Arc::new(db))
    }

    #[test]
    fn test_account_operations() {
        let (_temp_dir, db) = create_test_db();
        let store = StateStore::new(db).unwrap();
        
        let address = [1u8; 20];
        let state = AccountState {
            balance: 1000,
            nonce: 5,
            code_hash: None,
            storage_root: [0; 32],
        };
        
        // Test set and get
        store.set_account(&address, &state).unwrap();
        let retrieved = store.get_account(&address).unwrap();
        assert_eq!(retrieved, Some(state));
        
        // Test balance operations
        assert_eq!(store.get_balance(&address).unwrap(), 1000);
        store.set_balance(&address, 2000).unwrap();
        assert_eq!(store.get_balance(&address).unwrap(), 2000);
    }

    #[test]
    fn test_transfer() {
        let (_temp_dir, db) = create_test_db();
        let store = StateStore::new(db).unwrap();
        
        let from = [1u8; 20];
        let to = [2u8; 20];
        
        // Set initial balances
        store.set_balance(&from, 1000).unwrap();
        store.set_balance(&to, 500).unwrap();
        
        // Transfer
        store.transfer(&from, &to, 300).unwrap();
        
        // Check final balances
        assert_eq!(store.get_balance(&from).unwrap(), 700);
        assert_eq!(store.get_balance(&to).unwrap(), 800);
    }
}
