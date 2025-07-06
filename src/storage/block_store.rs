use anyhow::{Result, anyhow};
use rocksdb::{DB, IteratorMode};
use std::sync::Arc;
use crate::types::{Block, Hash, BlockHeight};

const CF_BLOCKS: &str = "blocks";

/// Block storage implementation
pub struct BlockStore {
    db: Arc<DB>,
}

impl BlockStore {
    pub fn new(db: Arc<DB>) -> Result<Self> {
        Ok(Self { db })
    }

    /// Store a block by height
    pub fn put_block(&self, block: &Block) -> Result<()> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let key = self.height_key(block.header.height);
        let serialized = bincode::serialize(block)
            .map_err(|e| anyhow!("Failed to serialize block: {}", e))?;
        
        self.db.put_cf(cf, &key, &serialized)
            .map_err(|e| anyhow!("Failed to store block: {}", e))?;

        // Also store by hash for quick lookups
        let hash_key = self.hash_key(&block.hash());
        self.db.put_cf(cf, &hash_key, &key)
            .map_err(|e| anyhow!("Failed to store block hash index: {}", e))?;

        Ok(())
    }

    /// Get a block by height
    pub fn get_block(&self, height: BlockHeight) -> Result<Option<Block>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let key = self.height_key(height);
        
        match self.db.get_cf(cf, &key)? {
            Some(data) => {
                let block = bincode::deserialize(&data)
                    .map_err(|e| anyhow!("Failed to deserialize block: {}", e))?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Get a block by hash
    pub fn get_block_by_hash(&self, hash: &Hash) -> Result<Option<Block>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let hash_key = self.hash_key(hash);
        
        // First get the height key from hash index
        match self.db.get_cf(cf, &hash_key)? {
            Some(height_key) => {
                // Then get the block using the height key
                match self.db.get_cf(cf, &height_key)? {
                    Some(data) => {
                        let block = bincode::deserialize(&data)
                            .map_err(|e| anyhow!("Failed to deserialize block: {}", e))?;
                        Ok(Some(block))
                    }
                    None => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Check if a block exists at given height
    pub fn has_block(&self, height: BlockHeight) -> Result<bool> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let key = self.height_key(height);
        Ok(self.db.get_cf(cf, &key)?.is_some())
    }

    /// Get block range
    pub fn get_blocks_range(&self, start: BlockHeight, end: BlockHeight) -> Result<Vec<Block>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let mut blocks = Vec::new();
        
        for height in start..=end {
            if let Some(block) = self.get_block(height)? {
                blocks.push(block);
            }
        }
        
        Ok(blocks)
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> Result<Option<Block>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let iter = self.db.iterator_cf(cf, IteratorMode::End);
        
        for item in iter {
            let (key, value) = item?;
            
            // Skip hash index entries (they start with 'h')
            if key.starts_with(b"h") {
                continue;
            }
            
            let block = bincode::deserialize(&value)
                .map_err(|e| anyhow!("Failed to deserialize block: {}", e))?;
            return Ok(Some(block));
        }
        
        Ok(None)
    }

    /// Count total number of blocks
    pub fn count_blocks(&self) -> Result<u64> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let mut count = 0;
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        
        for item in iter {
            let (key, _) = item?;
            
            // Only count height-based keys (skip hash index entries)
            if !key.starts_with(b"h") {
                count += 1;
            }
        }
        
        Ok(count)
    }

    /// Delete a block (use with caution)
    pub fn delete_block(&self, height: BlockHeight) -> Result<()> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        // First get the block to find its hash
        if let Some(block) = self.get_block(height)? {
            let hash_key = self.hash_key(&block.hash());
            self.db.delete_cf(cf, &hash_key)?;
        }
        
        let key = self.height_key(height);
        self.db.delete_cf(cf, &key)
            .map_err(|e| anyhow!("Failed to delete block: {}", e))?;
        
        Ok(())
    }

    /// Get all block heights
    pub fn get_all_heights(&self) -> Result<Vec<BlockHeight>> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or_else(|| anyhow!("Blocks column family not found"))?;
        
        let mut heights = Vec::new();
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        
        for item in iter {
            let (key, _) = item?;
            
            // Only process height-based keys (skip hash index entries)
            if !key.starts_with(b"h") {
                if let Ok(height) = self.parse_height_key(&key) {
                    heights.push(height);
                }
            }
        }
        
        heights.sort();
        Ok(heights)
    }

    /// Create a key for storing blocks by height
    fn height_key(&self, height: BlockHeight) -> Vec<u8> {
        format!("block_{:016}", height).into_bytes()
    }

    /// Create a key for storing block hash index
    fn hash_key(&self, hash: &Hash) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(b'h'); // Prefix for hash keys
        key.extend_from_slice(hash);
        key
    }

    /// Parse height from height key
    fn parse_height_key(&self, key: &[u8]) -> Result<BlockHeight> {
        let key_str = String::from_utf8_lossy(key);
        if let Some(height_str) = key_str.strip_prefix("block_") {
            height_str.parse::<BlockHeight>()
                .map_err(|e| anyhow!("Failed to parse height from key: {}", e))
        } else {
            Err(anyhow!("Invalid height key format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use rocksdb::{DB, Options, ColumnFamilyDescriptor};
    use crate::types::{BlockHeader, ValidatorSignature};

    fn create_test_db() -> (TempDir, Arc<DB>) {
        let temp_dir = TempDir::new().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_BLOCKS, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, temp_dir.path(), cfs).unwrap();
        (temp_dir, Arc::new(db))
    }

    fn create_test_block(height: BlockHeight) -> Block {
        Block {
            header: BlockHeader {
                height,
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
    fn test_block_store_operations() {
        let (_temp_dir, db) = create_test_db();
        let store = BlockStore::new(db).unwrap();
        
        let block = create_test_block(1);
        
        // Test storing and retrieving
        store.put_block(&block).unwrap();
        let retrieved = store.get_block(1).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), block);
        
        // Test has_block
        assert!(store.has_block(1).unwrap());
        assert!(!store.has_block(2).unwrap());
        
        // Test count
        assert_eq!(store.count_blocks().unwrap(), 1);
    }

    #[test]
    fn test_block_store_by_hash() {
        let (_temp_dir, db) = create_test_db();
        let store = BlockStore::new(db).unwrap();
        
        let block = create_test_block(1);
        let hash = block.hash();
        
        store.put_block(&block).unwrap();
        let retrieved = store.get_block_by_hash(&hash).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), block);
    }
}
