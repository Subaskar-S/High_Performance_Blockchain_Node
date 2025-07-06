use anyhow::{Result, anyhow};
use rocksdb::{DB, Options, ColumnFamily, ColumnFamilyDescriptor};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::sync::Arc;
use crate::types::{Block, Transaction, Hash, BlockHeight, NodeState};

pub mod block_store;
pub mod state_store;
pub mod transaction_store;

use block_store::BlockStore;
use state_store::StateStore;
use transaction_store::TransactionStore;

/// Column family names
const CF_BLOCKS: &str = "blocks";
const CF_TRANSACTIONS: &str = "transactions";
const CF_STATE: &str = "state";
const CF_METADATA: &str = "metadata";

/// Main storage interface for the blockchain node
pub struct Storage {
    db: Arc<DB>,
    block_store: BlockStore,
    state_store: StateStore,
    transaction_store: TransactionStore,
}

impl Storage {
    /// Create a new storage instance
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Define column families
        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_BLOCKS, Options::default()),
            ColumnFamilyDescriptor::new(CF_TRANSACTIONS, Options::default()),
            ColumnFamilyDescriptor::new(CF_STATE, Options::default()),
            ColumnFamilyDescriptor::new(CF_METADATA, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, db_path, cfs)
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;
        
        let db = Arc::new(db);

        let block_store = BlockStore::new(db.clone())?;
        let state_store = StateStore::new(db.clone())?;
        let transaction_store = TransactionStore::new(db.clone())?;

        Ok(Self {
            db,
            block_store,
            state_store,
            transaction_store,
        })
    }

    /// Get block store
    pub fn blocks(&self) -> &BlockStore {
        &self.block_store
    }

    /// Get state store
    pub fn state(&self) -> &StateStore {
        &self.state_store
    }

    /// Get transaction store
    pub fn transactions(&self) -> &TransactionStore {
        &self.transaction_store
    }

    /// Store a complete block with all its data
    pub fn store_block(&self, block: &Block) -> Result<()> {
        // Store the block
        self.block_store.put_block(block)?;

        // Store all transactions in the block
        for tx in &block.transactions {
            self.transaction_store.put_transaction(tx)?;
        }

        // Update latest block height
        self.put_metadata("latest_height", &block.header.height)?;

        Ok(())
    }

    /// Get the latest block height
    pub fn get_latest_height(&self) -> Result<Option<BlockHeight>> {
        self.get_metadata("latest_height")
    }

    /// Get the genesis block hash
    pub fn get_genesis_hash(&self) -> Result<Option<Hash>> {
        self.get_metadata("genesis_hash")
    }

    /// Set the genesis block hash
    pub fn set_genesis_hash(&self, hash: &Hash) -> Result<()> {
        self.put_metadata("genesis_hash", hash)
    }

    /// Store metadata
    pub fn put_metadata<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let cf = self.db.cf_handle(CF_METADATA)
            .ok_or_else(|| anyhow!("Metadata column family not found"))?;
        
        let serialized = bincode::serialize(value)
            .map_err(|e| anyhow!("Failed to serialize metadata: {}", e))?;
        
        self.db.put_cf(cf, key.as_bytes(), &serialized)
            .map_err(|e| anyhow!("Failed to store metadata: {}", e))?;

        Ok(())
    }

    /// Get metadata
    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let cf = self.db.cf_handle(CF_METADATA)
            .ok_or_else(|| anyhow!("Metadata column family not found"))?;
        
        match self.db.get_cf(cf, key.as_bytes())? {
            Some(data) => {
                let value = bincode::deserialize(&data)
                    .map_err(|e| anyhow!("Failed to deserialize metadata: {}", e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let latest_height = self.get_latest_height()?.unwrap_or(0);
        let total_blocks = self.block_store.count_blocks()?;
        let total_transactions = self.transaction_store.count_transactions()?;

        Ok(StorageStats {
            latest_height,
            total_blocks,
            total_transactions,
            db_size_bytes: self.estimate_db_size()?,
        })
    }

    /// Estimate database size
    fn estimate_db_size(&self) -> Result<u64> {
        // This is a simplified estimation
        // In production, you'd want more accurate size calculation
        Ok(0) // Placeholder
    }

    /// Compact the database
    pub fn compact(&self) -> Result<()> {
        self.db.compact_range::<&[u8], &[u8]>(None, None);
        Ok(())
    }

    /// Create a backup of the database
    pub fn backup<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        // Implementation would depend on RocksDB backup API
        // This is a placeholder
        Ok(())
    }

    /// Close the database
    pub fn close(self) -> Result<()> {
        // RocksDB will be closed when Arc is dropped
        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub latest_height: BlockHeight,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub db_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::types::{BlockHeader, Transaction, ValidatorSignature};
    use uuid::Uuid;

    fn create_test_block() -> Block {
        Block {
            header: BlockHeader {
                height: 1,
                previous_hash: [0; 32],
                merkle_root: [0; 32],
                state_root: [0; 32],
                timestamp: 1234567890,
                proposer: "test-node".to_string(),
                round: 1,
                view: 1,
            },
            transactions: vec![],
            signatures: vec![],
        }
    }

    #[test]
    fn test_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();
        
        // Test basic operations
        let stats = storage.get_stats().unwrap();
        assert_eq!(stats.latest_height, 0);
    }

    #[test]
    fn test_block_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();
        
        let block = create_test_block();
        storage.store_block(&block).unwrap();
        
        let retrieved = storage.blocks().get_block(1).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), block);
    }
}
