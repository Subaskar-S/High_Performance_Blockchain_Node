use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use anyhow::{Result, anyhow};
use uuid::Uuid;

use crate::types::{
    Block, ConsensusMessage, VoteType, Hash, NodeId, BlockHeight,
    ValidatorSignature, NetworkMessage, MessagePayload
};
use crate::storage::Storage;
use crate::mempool::Mempool;

pub mod pbft;
pub mod leader_election;
pub mod view_change;

pub use pbft::PbftEngine;
pub use leader_election::LeaderElection;
pub use view_change::ViewChangeManager;

/// Consensus engine configuration
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub node_id: NodeId,
    pub is_validator: bool,
    pub validator_set: Vec<NodeId>,
    pub block_time_ms: u64,
    pub view_timeout_ms: u64,
    pub max_block_size: usize,
    pub max_transactions_per_block: usize,
}

/// Consensus state
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusState {
    Idle,
    Proposing,
    Preparing,
    Committing,
    ViewChanging,
}

/// Consensus statistics
#[derive(Debug, Clone, Default)]
pub struct ConsensusStats {
    pub current_height: BlockHeight,
    pub current_view: u64,
    pub current_round: u64,
    pub total_blocks_proposed: u64,
    pub total_blocks_committed: u64,
    pub total_view_changes: u64,
    pub avg_consensus_time_ms: f64,
    pub current_leader: Option<NodeId>,
}

/// Main consensus engine implementing Byzantine Fault Tolerant consensus
pub struct ConsensusEngine {
    config: ConsensusConfig,
    state: Arc<RwLock<ConsensusState>>,
    current_view: Arc<RwLock<u64>>,
    current_round: Arc<RwLock<u64>>,
    current_height: Arc<RwLock<BlockHeight>>,
    
    // Core components
    pbft_engine: PbftEngine,
    leader_election: LeaderElection,
    view_change_manager: ViewChangeManager,
    
    // Storage and mempool
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    
    // Message handling
    message_sender: mpsc::UnboundedSender<NetworkMessage>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<NetworkMessage>>>>,
    
    // Consensus data
    pending_blocks: Arc<RwLock<HashMap<Hash, Block>>>,
    votes: Arc<RwLock<HashMap<(u64, u64, Hash), HashMap<NodeId, ConsensusMessage>>>>,
    
    // Statistics
    stats: Arc<RwLock<ConsensusStats>>,
    
    // Timing
    last_block_time: Arc<RwLock<Instant>>,
    view_timeout: Arc<RwLock<Option<tokio::time::Instant>>>,
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub fn new(
        config: ConsensusConfig,
        storage: Arc<Storage>,
        mempool: Arc<Mempool>,
    ) -> Result<Self> {
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        let pbft_engine = PbftEngine::new(config.clone())?;
        let leader_election = LeaderElection::new(config.validator_set.clone());
        let view_change_manager = ViewChangeManager::new(config.clone());
        
        // Initialize current state from storage
        let current_height = storage.get_latest_height()?.unwrap_or(0);
        
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(ConsensusState::Idle)),
            current_view: Arc::new(RwLock::new(0)),
            current_round: Arc::new(RwLock::new(0)),
            current_height: Arc::new(RwLock::new(current_height)),
            
            pbft_engine,
            leader_election,
            view_change_manager,
            
            storage,
            mempool,
            
            message_sender,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
            
            pending_blocks: Arc::new(RwLock::new(HashMap::new())),
            votes: Arc::new(RwLock::new(HashMap::new())),
            
            stats: Arc::new(RwLock::new(ConsensusStats::default())),
            
            last_block_time: Arc::new(RwLock::new(Instant::now())),
            view_timeout: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the consensus engine
    pub async fn start(&self) -> Result<()> {
        if !self.config.is_validator {
            return self.start_observer_mode().await;
        }

        // Start message processing
        let message_receiver = {
            let mut receiver_guard = self.message_receiver.write().unwrap();
            receiver_guard.take().ok_or_else(|| anyhow!("Consensus already started"))?
        };

        // Spawn message processing task
        let engine = self.clone();
        tokio::spawn(async move {
            engine.process_messages(message_receiver).await;
        });

        // Start consensus rounds
        self.start_consensus_loop().await
    }

    /// Start consensus loop for validators
    async fn start_consensus_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(self.config.block_time_ms));
        
        loop {
            interval.tick().await;
            
            // Check if we should propose a new block
            if self.should_propose_block().await? {
                if let Err(e) = self.propose_block().await {
                    tracing::error!("Failed to propose block: {}", e);
                }
            }
            
            // Check for view timeout
            if self.should_trigger_view_change().await {
                if let Err(e) = self.trigger_view_change().await {
                    tracing::error!("Failed to trigger view change: {}", e);
                }
            }
        }
    }

    /// Start observer mode (non-validator)
    async fn start_observer_mode(&self) -> Result<()> {
        let message_receiver = {
            let mut receiver_guard = self.message_receiver.write().unwrap();
            receiver_guard.take().ok_or_else(|| anyhow!("Consensus already started"))?
        };

        // Only process messages, don't participate in consensus
        self.process_messages(message_receiver).await;
        Ok(())
    }

    /// Process incoming consensus messages
    async fn process_messages(&self, mut receiver: mpsc::UnboundedReceiver<NetworkMessage>) {
        while let Some(message) = receiver.recv().await {
            if let MessagePayload::Consensus(consensus_msg) = message.payload {
                if let Err(e) = self.handle_consensus_message(consensus_msg).await {
                    tracing::error!("Failed to handle consensus message: {}", e);
                }
            }
        }
    }

    /// Handle incoming consensus message
    async fn handle_consensus_message(&self, message: ConsensusMessage) -> Result<()> {
        match message {
            ConsensusMessage::Propose { block, round, view } => {
                self.handle_propose_message(block, round, view).await
            }
            ConsensusMessage::Vote { block_hash, vote_type, round, view, validator_id, signature } => {
                self.handle_vote_message(block_hash, vote_type, round, view, validator_id, signature).await
            }
            ConsensusMessage::ViewChange { new_view, validator_id, signature } => {
                self.handle_view_change_message(new_view, validator_id, signature).await
            }
            ConsensusMessage::NewView { view, view_change_messages } => {
                self.handle_new_view_message(view, view_change_messages).await
            }
        }
    }

    /// Handle block proposal message
    async fn handle_propose_message(&self, block: Block, round: u64, view: u64) -> Result<()> {
        let current_view = *self.current_view.read().unwrap();
        let current_round = *self.current_round.read().unwrap();
        
        // Check if proposal is for current view and round
        if view != current_view || round != current_round {
            return Ok(()); // Ignore outdated proposals
        }

        // Validate the proposed block
        if !self.validate_proposed_block(&block).await? {
            tracing::warn!("Received invalid block proposal");
            return Ok(());
        }

        // Store the block
        let block_hash = block.hash();
        {
            let mut pending_blocks = self.pending_blocks.write().unwrap();
            pending_blocks.insert(block_hash, block);
        }

        // Send prepare vote
        self.send_vote(block_hash, VoteType::Prepare, round, view).await?;
        
        // Update state
        {
            let mut state = self.state.write().unwrap();
            *state = ConsensusState::Preparing;
        }

        Ok(())
    }

    /// Handle vote message
    async fn handle_vote_message(
        &self,
        block_hash: Hash,
        vote_type: VoteType,
        round: u64,
        view: u64,
        validator_id: NodeId,
        signature: crate::types::Signature,
    ) -> Result<()> {
        let current_view = *self.current_view.read().unwrap();
        let current_round = *self.current_round.read().unwrap();
        
        // Check if vote is for current view and round
        if view != current_view || round != current_round {
            return Ok(()); // Ignore outdated votes
        }

        // Verify validator is in validator set
        if !self.config.validator_set.contains(&validator_id) {
            return Ok(()); // Ignore votes from non-validators
        }

        // Store the vote
        let vote_key = (view, round, block_hash);
        let vote_message = ConsensusMessage::Vote {
            block_hash,
            vote_type: vote_type.clone(),
            round,
            view,
            validator_id: validator_id.clone(),
            signature,
        };

        {
            let mut votes = self.votes.write().unwrap();
            votes.entry(vote_key)
                .or_insert_with(HashMap::new)
                .insert(validator_id, vote_message);
        }

        // Check if we have enough votes to proceed
        self.check_vote_threshold(block_hash, vote_type, round, view).await?;

        Ok(())
    }

    /// Check if we have enough votes to proceed to next phase
    async fn check_vote_threshold(
        &self,
        block_hash: Hash,
        vote_type: VoteType,
        round: u64,
        view: u64,
    ) -> Result<()> {
        let vote_key = (view, round, block_hash);
        let required_votes = (self.config.validator_set.len() * 2 / 3) + 1;
        
        let vote_count = {
            let votes = self.votes.read().unwrap();
            votes.get(&vote_key)
                .map(|vote_map| {
                    vote_map.values()
                        .filter(|msg| {
                            if let ConsensusMessage::Vote { vote_type: msg_vote_type, .. } = msg {
                                msg_vote_type == &vote_type
                            } else {
                                false
                            }
                        })
                        .count()
                })
                .unwrap_or(0)
        };

        if vote_count >= required_votes {
            match vote_type {
                VoteType::Prepare => {
                    // Move to commit phase
                    self.send_vote(block_hash, VoteType::Commit, round, view).await?;
                    
                    let mut state = self.state.write().unwrap();
                    *state = ConsensusState::Committing;
                }
                VoteType::Commit => {
                    // Commit the block
                    self.commit_block(block_hash).await?;
                }
            }
        }

        Ok(())
    }

    /// Commit a block to the blockchain
    async fn commit_block(&self, block_hash: Hash) -> Result<()> {
        let block = {
            let pending_blocks = self.pending_blocks.read().unwrap();
            pending_blocks.get(&block_hash).cloned()
                .ok_or_else(|| anyhow!("Block not found in pending blocks"))?
        };

        // Store the block
        self.storage.store_block(&block)?;

        // Update current height
        {
            let mut current_height = self.current_height.write().unwrap();
            *current_height = block.header.height;
        }

        // Remove committed transactions from mempool
        for tx in &block.transactions {
            let _ = self.mempool.remove_transaction(&tx.id);
        }

        // Clean up
        {
            let mut pending_blocks = self.pending_blocks.write().unwrap();
            pending_blocks.remove(&block_hash);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.current_height = block.header.height;
            stats.total_blocks_committed += 1;
        }

        // Reset state for next round
        {
            let mut state = self.state.write().unwrap();
            *state = ConsensusState::Idle;
        }

        {
            let mut current_round = self.current_round.write().unwrap();
            *current_round += 1;
        }

        tracing::info!("Block committed at height {}", block.header.height);

        Ok(())
    }

    /// Check if this node should propose a block
    async fn should_propose_block(&self) -> Result<bool> {
        let current_view = *self.current_view.read().unwrap();
        let current_round = *self.current_round.read().unwrap();
        
        // Check if we are the leader for current view
        let leader = self.leader_election.get_leader(current_view);
        if leader != self.config.node_id {
            return Ok(false);
        }

        // Check if we're in idle state
        let state = self.state.read().unwrap();
        if *state != ConsensusState::Idle {
            return Ok(false);
        }

        // Check if enough time has passed since last block
        let last_block_time = *self.last_block_time.read().unwrap();
        let elapsed = last_block_time.elapsed();
        if elapsed < Duration::from_millis(self.config.block_time_ms) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Propose a new block
    async fn propose_block(&self) -> Result<()> {
        let current_view = *self.current_view.read().unwrap();
        let current_round = *self.current_round.read().unwrap();
        let current_height = *self.current_height.read().unwrap();

        // Get transactions from mempool
        let transactions = self.mempool.get_next_batch(
            self.config.max_transactions_per_block,
            self.config.max_block_size,
        )?;

        if transactions.is_empty() {
            return Ok(()); // No transactions to include
        }

        // Create new block
        let block = self.create_block(transactions, current_height + 1, current_view, current_round).await?;

        // Broadcast proposal
        let proposal = ConsensusMessage::Propose {
            block: block.clone(),
            round: current_round,
            view: current_view,
        };

        self.broadcast_consensus_message(proposal).await?;

        // Store our own proposal
        let block_hash = block.hash();
        {
            let mut pending_blocks = self.pending_blocks.write().unwrap();
            pending_blocks.insert(block_hash, block);
        }

        // Update state
        {
            let mut state = self.state.write().unwrap();
            *state = ConsensusState::Proposing;
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_blocks_proposed += 1;
        }

        tracing::info!("Proposed block at height {}", current_height + 1);

        Ok(())
    }

    /// Create a new block
    async fn create_block(
        &self,
        transactions: Vec<crate::types::Transaction>,
        height: BlockHeight,
        view: u64,
        round: u64,
    ) -> Result<Block> {
        // Get previous block hash
        let previous_hash = if height == 0 {
            [0; 32] // Genesis block
        } else {
            let prev_block = self.storage.blocks().get_block(height - 1)?
                .ok_or_else(|| anyhow!("Previous block not found"))?;
            prev_block.hash()
        };

        // Create block header
        let mut block = Block {
            header: crate::types::BlockHeader {
                height,
                previous_hash,
                merkle_root: [0; 32], // Will be calculated
                state_root: [0; 32], // Simplified
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                proposer: self.config.node_id.clone(),
                round,
                view,
            },
            transactions,
            signatures: vec![],
        };

        // Calculate merkle root
        block.header.merkle_root = block.calculate_merkle_root();

        Ok(block)
    }

    /// Send a vote message
    async fn send_vote(
        &self,
        block_hash: Hash,
        vote_type: VoteType,
        round: u64,
        view: u64,
    ) -> Result<()> {
        let vote = ConsensusMessage::Vote {
            block_hash,
            vote_type,
            round,
            view,
            validator_id: self.config.node_id.clone(),
            signature: [0; 64], // Simplified signature
        };

        self.broadcast_consensus_message(vote).await
    }

    /// Broadcast consensus message to all validators
    async fn broadcast_consensus_message(&self, message: ConsensusMessage) -> Result<()> {
        let network_message = NetworkMessage::new(
            self.config.node_id.clone(),
            MessagePayload::Consensus(message),
        );

        // Send through network layer (simplified)
        self.message_sender.send(network_message)
            .map_err(|e| anyhow!("Failed to send message: {}", e))?;

        Ok(())
    }

    /// Validate a proposed block
    async fn validate_proposed_block(&self, block: &Block) -> Result<bool> {
        // Use the validation module
        // This is a simplified validation
        Ok(block.verify())
    }

    /// Handle view change message
    async fn handle_view_change_message(
        &self,
        new_view: u64,
        validator_id: NodeId,
        signature: crate::types::Signature,
    ) -> Result<()> {
        self.view_change_manager.handle_view_change(new_view, validator_id, signature).await
    }

    /// Handle new view message
    async fn handle_new_view_message(
        &self,
        view: u64,
        view_change_messages: Vec<ConsensusMessage>,
    ) -> Result<()> {
        self.view_change_manager.handle_new_view(view, view_change_messages).await
    }

    /// Check if view change should be triggered
    async fn should_trigger_view_change(&self) -> bool {
        // Check for timeout
        if let Some(timeout) = *self.view_timeout.read().unwrap() {
            return tokio::time::Instant::now() > timeout;
        }
        false
    }

    /// Trigger view change
    async fn trigger_view_change(&self) -> Result<()> {
        let current_view = *self.current_view.read().unwrap();
        let new_view = current_view + 1;

        self.view_change_manager.trigger_view_change(new_view).await?;

        // Update view
        {
            let mut view = self.current_view.write().unwrap();
            *view = new_view;
        }

        // Update state
        {
            let mut state = self.state.write().unwrap();
            *state = ConsensusState::ViewChanging;
        }

        Ok(())
    }

    /// Get current consensus statistics
    pub fn get_stats(&self) -> ConsensusStats {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Get message sender for network layer
    pub fn get_message_sender(&self) -> mpsc::UnboundedSender<NetworkMessage> {
        self.message_sender.clone()
    }
}

// Implement Clone for ConsensusEngine (needed for tokio::spawn)
impl Clone for ConsensusEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: self.state.clone(),
            current_view: self.current_view.clone(),
            current_round: self.current_round.clone(),
            current_height: self.current_height.clone(),
            pbft_engine: self.pbft_engine.clone(),
            leader_election: self.leader_election.clone(),
            view_change_manager: self.view_change_manager.clone(),
            storage: self.storage.clone(),
            mempool: self.mempool.clone(),
            message_sender: self.message_sender.clone(),
            message_receiver: self.message_receiver.clone(),
            pending_blocks: self.pending_blocks.clone(),
            votes: self.votes.clone(),
            stats: self.stats.clone(),
            last_block_time: self.last_block_time.clone(),
            view_timeout: self.view_timeout.clone(),
        }
    }
}
