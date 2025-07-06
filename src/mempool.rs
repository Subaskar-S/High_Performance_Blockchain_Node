use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use crate::types::{Transaction, Address, Hash};

/// Transaction wrapper for priority queue ordering
#[derive(Debug, Clone)]
struct PriorityTransaction {
    transaction: Transaction,
    priority_score: u64,
    insertion_time: u64,
}

impl PartialEq for PriorityTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.transaction.id == other.transaction.id
    }
}

impl Eq for PriorityTransaction {}

impl PartialOrd for PriorityTransaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTransaction {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then by insertion time (FIFO for same priority)
        match self.priority_score.cmp(&other.priority_score) {
            Ordering::Equal => other.insertion_time.cmp(&self.insertion_time),
            other => other,
        }
    }
}

/// Transaction pool configuration
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    pub max_size: usize,
    pub max_per_account: usize,
    pub min_fee: u64,
    pub max_tx_size: usize,
    pub cleanup_interval_secs: u64,
    pub max_age_secs: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            max_per_account: 100,
            min_fee: 1,
            max_tx_size: 1024 * 1024, // 1MB
            cleanup_interval_secs: 60,
            max_age_secs: 3600, // 1 hour
        }
    }
}

/// High-performance transaction mempool with priority queue
pub struct Mempool {
    config: MempoolConfig,
    // Priority queue for transaction ordering
    priority_queue: Arc<RwLock<BinaryHeap<PriorityTransaction>>>,
    // Fast lookup by transaction ID
    transactions: Arc<RwLock<HashMap<Uuid, Transaction>>>,
    // Track transactions by sender for nonce validation
    by_sender: Arc<RwLock<HashMap<Address, Vec<Uuid>>>>,
    // Track transaction hashes to prevent duplicates
    tx_hashes: Arc<RwLock<HashSet<Hash>>>,
    // Statistics
    stats: Arc<RwLock<MempoolStats>>,
    // Insertion counter for FIFO ordering
    insertion_counter: Arc<RwLock<u64>>,
}

/// Mempool statistics
#[derive(Debug, Clone, Default)]
pub struct MempoolStats {
    pub total_transactions: usize,
    pub total_added: u64,
    pub total_removed: u64,
    pub total_rejected: u64,
    pub avg_fee: u64,
    pub pending_by_fee: HashMap<u64, usize>,
}

impl Mempool {
    /// Create a new mempool
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            by_sender: Arc::new(RwLock::new(HashMap::new())),
            tx_hashes: Arc::new(RwLock::new(HashSet::new())),
            stats: Arc::new(RwLock::new(MempoolStats::default())),
            insertion_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// Add a transaction to the mempool
    pub fn add_transaction(&self, tx: Transaction) -> Result<bool> {
        // Basic validation
        if !self.validate_transaction(&tx)? {
            self.increment_rejected();
            return Ok(false);
        }

        let tx_hash = tx.hash();
        let tx_id = tx.id;
        let sender = tx.from;

        // Check for duplicates
        {
            let hashes = self.tx_hashes.read().unwrap();
            if hashes.contains(&tx_hash) {
                return Ok(false); // Already exists
            }
        }

        // Check mempool capacity
        if self.is_full() && !self.should_replace(&tx)? {
            self.increment_rejected();
            return Ok(false);
        }

        // Get insertion time and priority
        let insertion_time = {
            let mut counter = self.insertion_counter.write().unwrap();
            *counter += 1;
            *counter
        };

        let priority_score = self.calculate_priority(&tx);

        // Add to all data structures
        {
            let mut transactions = self.transactions.write().unwrap();
            let mut priority_queue = self.priority_queue.write().unwrap();
            let mut by_sender = self.by_sender.write().unwrap();
            let mut tx_hashes = self.tx_hashes.write().unwrap();

            // Add to main storage
            transactions.insert(tx_id, tx.clone());
            tx_hashes.insert(tx_hash);

            // Add to priority queue
            priority_queue.push(PriorityTransaction {
                transaction: tx.clone(),
                priority_score,
                insertion_time,
            });

            // Track by sender
            by_sender.entry(sender).or_insert_with(Vec::new).push(tx_id);

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.total_transactions = transactions.len();
            stats.total_added += 1;
        }

        // Clean up if necessary
        if self.is_full() {
            self.cleanup_low_priority()?;
        }

        Ok(true)
    }

    /// Remove a transaction from the mempool
    pub fn remove_transaction(&self, tx_id: &Uuid) -> Result<Option<Transaction>> {
        let mut transactions = self.transactions.write().unwrap();
        let mut by_sender = self.by_sender.write().unwrap();
        let mut tx_hashes = self.tx_hashes.write().unwrap();

        if let Some(tx) = transactions.remove(tx_id) {
            // Remove from hash set
            tx_hashes.remove(&tx.hash());

            // Remove from sender tracking
            if let Some(sender_txs) = by_sender.get_mut(&tx.from) {
                sender_txs.retain(|id| id != tx_id);
                if sender_txs.is_empty() {
                    by_sender.remove(&tx.from);
                }
            }

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.total_transactions = transactions.len();
            stats.total_removed += 1;

            // Note: We don't remove from priority queue immediately for performance
            // The queue will be cleaned up during next iteration

            Ok(Some(tx))
        } else {
            Ok(None)
        }
    }

    /// Get next batch of transactions for block creation
    pub fn get_next_batch(&self, max_count: usize, max_size: usize) -> Result<Vec<Transaction>> {
        let mut batch = Vec::new();
        let mut total_size = 0;
        let mut processed_senders = HashSet::new();

        let transactions = self.transactions.read().unwrap();
        let mut priority_queue = self.priority_queue.write().unwrap();
        let mut temp_queue = BinaryHeap::new();

        // Extract transactions from priority queue
        while let Some(priority_tx) = priority_queue.pop() {
            // Check if transaction still exists (might have been removed)
            if !transactions.contains_key(&priority_tx.transaction.id) {
                continue; // Skip removed transactions
            }

            let tx = &priority_tx.transaction;
            let tx_size = bincode::serialize(tx).unwrap_or_default().len();

            // Check batch limits
            if batch.len() >= max_count || total_size + tx_size > max_size {
                temp_queue.push(priority_tx);
                break;
            }

            // Check nonce ordering for sender
            if self.is_valid_nonce_order(&tx, &processed_senders)? {
                batch.push(tx.clone());
                total_size += tx_size;
                processed_senders.insert(tx.from);
            }

            temp_queue.push(priority_tx);
        }

        // Restore remaining transactions to priority queue
        while let Some(priority_tx) = temp_queue.pop() {
            priority_queue.push(priority_tx);
        }

        Ok(batch)
    }

    /// Get transaction by ID
    pub fn get_transaction(&self, tx_id: &Uuid) -> Option<Transaction> {
        let transactions = self.transactions.read().unwrap();
        transactions.get(tx_id).cloned()
    }

    /// Get transactions by sender
    pub fn get_transactions_by_sender(&self, sender: &Address) -> Vec<Transaction> {
        let transactions = self.transactions.read().unwrap();
        let by_sender = self.by_sender.read().unwrap();

        if let Some(tx_ids) = by_sender.get(sender) {
            tx_ids.iter()
                .filter_map(|id| transactions.get(id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if mempool contains transaction
    pub fn contains(&self, tx_id: &Uuid) -> bool {
        let transactions = self.transactions.read().unwrap();
        transactions.contains_key(tx_id)
    }

    /// Get current mempool size
    pub fn size(&self) -> usize {
        let transactions = self.transactions.read().unwrap();
        transactions.len()
    }

    /// Check if mempool is full
    pub fn is_full(&self) -> bool {
        self.size() >= self.config.max_size
    }

    /// Get mempool statistics
    pub fn get_stats(&self) -> MempoolStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Clear all transactions
    pub fn clear(&self) -> Result<()> {
        let mut transactions = self.transactions.write().unwrap();
        let mut priority_queue = self.priority_queue.write().unwrap();
        let mut by_sender = self.by_sender.write().unwrap();
        let mut tx_hashes = self.tx_hashes.write().unwrap();

        transactions.clear();
        priority_queue.clear();
        by_sender.clear();
        tx_hashes.clear();

        let mut stats = self.stats.write().unwrap();
        stats.total_transactions = 0;

        Ok(())
    }

    /// Validate transaction before adding to mempool
    fn validate_transaction(&self, tx: &Transaction) -> Result<bool> {
        // Check minimum fee
        if tx.fee < self.config.min_fee {
            return Ok(false);
        }

        // Check transaction size
        let tx_size = bincode::serialize(tx).unwrap_or_default().len();
        if tx_size > self.config.max_tx_size {
            return Ok(false);
        }

        // Check per-account limit
        let by_sender = self.by_sender.read().unwrap();
        if let Some(sender_txs) = by_sender.get(&tx.from) {
            if sender_txs.len() >= self.config.max_per_account {
                return Ok(false);
            }
        }

        // Basic transaction validation
        if !tx.verify() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Calculate transaction priority score
    fn calculate_priority(&self, tx: &Transaction) -> u64 {
        // Simple fee-based priority (can be enhanced with other factors)
        tx.fee
    }

    /// Check if transaction should replace existing ones
    fn should_replace(&self, tx: &Transaction) -> Result<bool> {
        // For now, only replace if fee is significantly higher
        // This is a simplified replacement strategy
        Ok(tx.fee > self.config.min_fee * 2)
    }

    /// Clean up low priority transactions when mempool is full
    fn cleanup_low_priority(&self) -> Result<()> {
        let target_size = (self.config.max_size as f64 * 0.9) as usize;
        let current_size = self.size();

        if current_size <= target_size {
            return Ok(());
        }

        let to_remove = current_size - target_size;
        let mut removed_count = 0;

        // Remove lowest priority transactions
        let mut priority_queue = self.priority_queue.write().unwrap();
        let mut temp_queue = BinaryHeap::new();

        while let Some(priority_tx) = priority_queue.pop() {
            if removed_count < to_remove {
                // Remove this transaction
                self.remove_transaction(&priority_tx.transaction.id)?;
                removed_count += 1;
            } else {
                temp_queue.push(priority_tx);
            }
        }

        // Restore remaining transactions
        while let Some(priority_tx) = temp_queue.pop() {
            priority_queue.push(priority_tx);
        }

        Ok(())
    }

    /// Check if transaction nonce is valid for batch ordering
    fn is_valid_nonce_order(&self, tx: &Transaction, processed_senders: &HashSet<Address>) -> Result<bool> {
        // Simplified nonce validation - in production, this would be more sophisticated
        if processed_senders.contains(&tx.from) {
            return Ok(false); // Only one tx per sender per batch for simplicity
        }
        Ok(true)
    }

    /// Increment rejected transaction counter
    fn increment_rejected(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.total_rejected += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_transaction(from: Address, fee: u64) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            from,
            to: [2u8; 20],
            amount: 1000,
            fee,
            nonce: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            signature: [0u8; 64],
            data: vec![],
        }
    }

    #[test]
    fn test_mempool_basic_operations() {
        let mempool = Mempool::new(MempoolConfig::default());
        
        let tx = create_test_transaction([1u8; 20], 10);
        let tx_id = tx.id;
        
        // Add transaction
        assert!(mempool.add_transaction(tx.clone()).unwrap());
        assert_eq!(mempool.size(), 1);
        assert!(mempool.contains(&tx_id));
        
        // Get transaction
        let retrieved = mempool.get_transaction(&tx_id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, tx_id);
        
        // Remove transaction
        let removed = mempool.remove_transaction(&tx_id).unwrap();
        assert!(removed.is_some());
        assert_eq!(mempool.size(), 0);
    }

    #[test]
    fn test_mempool_priority_ordering() {
        let mempool = Mempool::new(MempoolConfig::default());
        
        // Add transactions with different fees
        let tx1 = create_test_transaction([1u8; 20], 5);
        let tx2 = create_test_transaction([2u8; 20], 15);
        let tx3 = create_test_transaction([3u8; 20], 10);
        
        mempool.add_transaction(tx1).unwrap();
        mempool.add_transaction(tx2.clone()).unwrap();
        mempool.add_transaction(tx3).unwrap();
        
        // Get batch - should return highest fee first
        let batch = mempool.get_next_batch(10, 1024 * 1024).unwrap();
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].id, tx2.id); // Highest fee should be first
    }
}
