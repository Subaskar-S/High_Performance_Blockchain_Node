use anyhow::{Result, anyhow};
use rocksdb::DB;
use std::sync::Arc;
use uuid::Uuid;
use crate::types::{Transaction, Hash, BlockHeight};

const CF_TRANSACTIONS: &str = "transactions";

/// Transaction storage implementation
pub struct TransactionStore {
    db: Arc<DB>,
}

impl TransactionStore {
    pub fn new(db: Arc<DB>) -> Result<Self> {
        Ok(Self { db })
    }

    /// Store a transaction
    pub fn put_transaction(&self, tx: &Transaction) -> Result<()> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let key = self.tx_key(&tx.id);
        let serialized = bincode::serialize(tx)
            .map_err(|e| anyhow!("Failed to serialize transaction: {}", e))?;
        
        self.db.put_cf(cf, &key, &serialized)
            .map_err(|e| anyhow!("Failed to store transaction: {}", e))?;

        // Also store by hash for quick lookups
        let hash_key = self.hash_key(&tx.hash());
        self.db.put_cf(cf, &hash_key, &key)
            .map_err(|e| anyhow!("Failed to store transaction hash index: {}", e))?;

        Ok(())
    }

    /// Get a transaction by ID
    pub fn get_transaction(&self, tx_id: &Uuid) -> Result<Option<Transaction>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let key = self.tx_key(tx_id);
        
        match self.db.get_cf(cf, &key)? {
            Some(data) => {
                let tx = bincode::deserialize(&data)
                    .map_err(|e| anyhow!("Failed to deserialize transaction: {}", e))?;
                Ok(Some(tx))
            }
            None => Ok(None),
        }
    }

    /// Get a transaction by hash
    pub fn get_transaction_by_hash(&self, hash: &Hash) -> Result<Option<Transaction>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let hash_key = self.hash_key(hash);
        
        // First get the transaction ID from hash index
        match self.db.get_cf(cf, &hash_key)? {
            Some(tx_key) => {
                // Then get the transaction using the ID key
                match self.db.get_cf(cf, &tx_key)? {
                    Some(data) => {
                        let tx = bincode::deserialize(&data)
                            .map_err(|e| anyhow!("Failed to deserialize transaction: {}", e))?;
                        Ok(Some(tx))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Check if a transaction exists
    pub fn has_transaction(&self, tx_id: &Uuid) -> Result<bool> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let key = self.tx_key(tx_id);
        Ok(self.db.get_cf(cf, &key)?.is_some())
    }

    /// Get transactions by sender address
    pub fn get_transactions_by_sender(&self, sender: &[u8; 20]) -> Result<Vec<Transaction>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let mut transactions = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, value) = item?;
            
            // Skip hash index entries
            if key.starts_with(b"h") {
                continue;
            }
            
            if let Ok(tx) = bincode::deserialize::<Transaction>(&value) {
                if tx.from == *sender {
                    transactions.push(tx);
                }
            }
        }
        
        // Sort by timestamp
        transactions.sort_by_key(|tx| tx.timestamp);
        Ok(transactions)
    }

    /// Get transactions by recipient address
    pub fn get_transactions_by_recipient(&self, recipient: &[u8; 20]) -> Result<Vec<Transaction>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let mut transactions = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, value) = item?;
            
            // Skip hash index entries
            if key.starts_with(b"h") {
                continue;
            }
            
            if let Ok(tx) = bincode::deserialize::<Transaction>(&value) {
                if tx.to == *recipient {
                    transactions.push(tx);
                }
            }
        }
        
        // Sort by timestamp
        transactions.sort_by_key(|tx| tx.timestamp);
        Ok(transactions)
    }

    /// Get recent transactions (last N transactions)
    pub fn get_recent_transactions(&self, limit: usize) -> Result<Vec<Transaction>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let mut transactions = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::End);
        
        for item in iter {
            let (key, value) = item?;
            
            // Skip hash index entries
            if key.starts_with(b"h") {
                continue;
            }
            
            if let Ok(tx) = bincode::deserialize::<Transaction>(&value) {
                transactions.push(tx);
                if transactions.len() >= limit {
                    break;
                }
            }
        }
        
        // Sort by timestamp (most recent first)
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(transactions)
    }

    /// Count total number of transactions
    pub fn count_transactions(&self) -> Result<u64> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let mut count = 0;
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, _) = item?;
            
            // Only count transaction keys (skip hash index entries)
            if !key.starts_with(b"h") {
                count += 1;
            }
        }
        
        Ok(count)
    }

    /// Delete a transaction (use with caution)
    pub fn delete_transaction(&self, tx_id: &Uuid) -> Result<()> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        // First get the transaction to find its hash
        if let Some(tx) = self.get_transaction(tx_id)? {
            let hash_key = self.hash_key(&tx.hash());
            self.db.delete_cf(cf, &hash_key)?;
        }
        
        let key = self.tx_key(tx_id);
        self.db.delete_cf(cf, &key)
            .map_err(|e| anyhow!("Failed to delete transaction: {}", e))?;
        
        Ok(())
    }

    /// Get all transaction IDs
    pub fn get_all_transaction_ids(&self) -> Result<Vec<Uuid>> {
        let cf = self.db.cf_handle(CF_TRANSACTIONS)
            .ok_or_else(|| anyhow!("Transactions column family not found"))?;
        
        let mut ids = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, _) = item?;
            
            // Only process transaction keys (skip hash index entries)
            if !key.starts_with(b"h") {
                if let Ok(id) = self.parse_tx_key(&key) {
                    ids.push(id);
                }
            }
        }
        
        Ok(ids)
    }

    /// Create a key for storing transactions by ID
    fn tx_key(&self, tx_id: &Uuid) -> Vec<u8> {
        format!("tx_{}", tx_id).into_bytes()
    }

    /// Create a key for storing transaction hash index
    fn hash_key(&self, hash: &Hash) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(b'h'); // Prefix for hash keys
        key.extend_from_slice(hash);
        key
    }

    /// Parse transaction ID from transaction key
    fn parse_tx_key(&self, key: &[u8]) -> Result<Uuid> {
        let key_str = String::from_utf8_lossy(key);
        if let Some(id_str) = key_str.strip_prefix("tx_") {
            Uuid::parse_str(id_str)
                .map_err(|e| anyhow!("Failed to parse UUID from key: {}", e))
        } else {
            Err(anyhow!("Invalid transaction key format"))
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
            ColumnFamilyDescriptor::new(CF_TRANSACTIONS, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, temp_dir.path(), cfs).unwrap();
        (temp_dir, Arc::new(db))
    }

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            from: [1u8; 20],
            to: [2u8; 20],
            amount: 1000,
            fee: 10,
            nonce: 1,
            timestamp: 1234567890,
            signature: [0u8; 64],
            data: vec![],
        }
    }

    #[test]
    fn test_transaction_store_operations() {
        let (_temp_dir, db) = create_test_db();
        let store = TransactionStore::new(db).unwrap();
        
        let tx = create_test_transaction();
        let tx_id = tx.id;
        
        // Test storing and retrieving
        store.put_transaction(&tx).unwrap();
        let retrieved = store.get_transaction(&tx_id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), tx);
        
        // Test has_transaction
        assert!(store.has_transaction(&tx_id).unwrap());
        
        // Test count
        assert_eq!(store.count_transactions().unwrap(), 1);
    }

    #[test]
    fn test_transaction_store_by_hash() {
        let (_temp_dir, db) = create_test_db();
        let store = TransactionStore::new(db).unwrap();
        
        let tx = create_test_transaction();
        let hash = tx.hash();
        
        store.put_transaction(&tx).unwrap();
        let retrieved = store.get_transaction_by_hash(&hash).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), tx);
    }
}
