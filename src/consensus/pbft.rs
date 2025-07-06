use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use crate::types::{ConsensusMessage, VoteType, Hash, NodeId, Block};
use super::ConsensusConfig;

/// PBFT (Practical Byzantine Fault Tolerance) consensus phases
#[derive(Debug, Clone, PartialEq)]
pub enum PbftPhase {
    PrePrepare,
    Prepare,
    Commit,
    Committed,
}

/// PBFT message log entry
#[derive(Debug, Clone)]
pub struct PbftLogEntry {
    pub view: u64,
    pub sequence: u64,
    pub block_hash: Hash,
    pub phase: PbftPhase,
    pub messages: HashMap<NodeId, ConsensusMessage>,
}

/// PBFT consensus engine implementing the three-phase protocol
#[derive(Clone)]
pub struct PbftEngine {
    config: ConsensusConfig,
    
    // PBFT state
    current_view: Arc<RwLock<u64>>,
    current_sequence: Arc<RwLock<u64>>,
    
    // Message logs for each consensus instance
    message_log: Arc<RwLock<HashMap<(u64, u64), PbftLogEntry>>>,
    
    // Prepared and committed certificates
    prepared_certificates: Arc<RwLock<HashSet<(u64, u64, Hash)>>>,
    committed_certificates: Arc<RwLock<HashSet<(u64, u64, Hash)>>>,
    
    // View change state
    view_change_votes: Arc<RwLock<HashMap<u64, HashSet<NodeId>>>>,
}

impl PbftEngine {
    /// Create a new PBFT engine
    pub fn new(config: ConsensusConfig) -> Result<Self> {
        Ok(Self {
            config,
            current_view: Arc::new(RwLock::new(0)),
            current_sequence: Arc::new(RwLock::new(0)),
            message_log: Arc::new(RwLock::new(HashMap::new())),
            prepared_certificates: Arc::new(RwLock::new(HashSet::new())),
            committed_certificates: Arc::new(RwLock::new(HashSet::new())),
            view_change_votes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Process a consensus message according to PBFT protocol
    pub fn process_message(&self, message: ConsensusMessage) -> Result<Vec<ConsensusMessage>> {
        match message {
            ConsensusMessage::Propose { block, round, view } => {
                self.handle_pre_prepare(block, round, view)
            }
            ConsensusMessage::Vote { block_hash, vote_type, round, view, validator_id, signature } => {
                match vote_type {
                    VoteType::Prepare => {
                        self.handle_prepare(block_hash, round, view, validator_id, signature)
                    }
                    VoteType::Commit => {
                        self.handle_commit(block_hash, round, view, validator_id, signature)
                    }
                }
            }
            ConsensusMessage::ViewChange { new_view, validator_id, signature } => {
                self.handle_view_change(new_view, validator_id, signature)
            }
            ConsensusMessage::NewView { view, view_change_messages } => {
                self.handle_new_view(view, view_change_messages)
            }
        }
    }

    /// Handle PRE-PREPARE message (Phase 1)
    fn handle_pre_prepare(&self, block: Block, sequence: u64, view: u64) -> Result<Vec<ConsensusMessage>> {
        let current_view = *self.current_view.read().unwrap();
        
        // Check if message is for current view
        if view != current_view {
            return Ok(vec![]); // Ignore messages from different views
        }

        // Verify the proposer is the primary for this view
        if !self.is_primary(&block.header.proposer, view) {
            return Err(anyhow!("Invalid proposer for view {}", view));
        }

        let block_hash = block.hash();
        let log_key = (view, sequence);

        // Create or update log entry
        {
            let mut message_log = self.message_log.write().unwrap();
            let entry = message_log.entry(log_key).or_insert_with(|| PbftLogEntry {
                view,
                sequence,
                block_hash,
                phase: PbftPhase::PrePrepare,
                messages: HashMap::new(),
            });

            // Store the PRE-PREPARE message
            entry.messages.insert(
                block.header.proposer.clone(),
                ConsensusMessage::Propose { block, round: sequence, view },
            );
        }

        // If we're a validator, send PREPARE message
        if self.config.is_validator && self.config.node_id != block.header.proposer {
            let prepare_message = ConsensusMessage::Vote {
                block_hash,
                vote_type: VoteType::Prepare,
                round: sequence,
                view,
                validator_id: self.config.node_id.clone(),
                signature: [0; 64], // Simplified signature
            };

            Ok(vec![prepare_message])
        } else {
            Ok(vec![])
        }
    }

    /// Handle PREPARE message (Phase 2)
    fn handle_prepare(
        &self,
        block_hash: Hash,
        sequence: u64,
        view: u64,
        validator_id: NodeId,
        signature: crate::types::Signature,
    ) -> Result<Vec<ConsensusMessage>> {
        let current_view = *self.current_view.read().unwrap();
        
        if view != current_view {
            return Ok(vec![]);
        }

        // Verify validator is in validator set
        if !self.config.validator_set.contains(&validator_id) {
            return Ok(vec![]);
        }

        let log_key = (view, sequence);

        // Store the PREPARE message
        {
            let mut message_log = self.message_log.write().unwrap();
            if let Some(entry) = message_log.get_mut(&log_key) {
                if entry.block_hash == block_hash {
                    entry.messages.insert(
                        validator_id.clone(),
                        ConsensusMessage::Vote {
                            block_hash,
                            vote_type: VoteType::Prepare,
                            round: sequence,
                            view,
                            validator_id: validator_id.clone(),
                            signature,
                        },
                    );
                    entry.phase = PbftPhase::Prepare;
                }
            }
        }

        // Check if we have enough PREPARE messages (2f+1)
        let prepare_count = self.count_prepare_messages(view, sequence, block_hash);
        let required_count = self.byzantine_threshold();

        if prepare_count >= required_count {
            // Mark as prepared
            {
                let mut prepared = self.prepared_certificates.write().unwrap();
                prepared.insert((view, sequence, block_hash));
            }

            // Send COMMIT message if we're a validator
            if self.config.is_validator {
                let commit_message = ConsensusMessage::Vote {
                    block_hash,
                    vote_type: VoteType::Commit,
                    round: sequence,
                    view,
                    validator_id: self.config.node_id.clone(),
                    signature: [0; 64], // Simplified signature
                };

                return Ok(vec![commit_message]);
            }
        }

        Ok(vec![])
    }

    /// Handle COMMIT message (Phase 3)
    fn handle_commit(
        &self,
        block_hash: Hash,
        sequence: u64,
        view: u64,
        validator_id: NodeId,
        signature: crate::types::Signature,
    ) -> Result<Vec<ConsensusMessage>> {
        let current_view = *self.current_view.read().unwrap();
        
        if view != current_view {
            return Ok(vec![]);
        }

        // Verify validator is in validator set
        if !self.config.validator_set.contains(&validator_id) {
            return Ok(vec![]);
        }

        let log_key = (view, sequence);

        // Store the COMMIT message
        {
            let mut message_log = self.message_log.write().unwrap();
            if let Some(entry) = message_log.get_mut(&log_key) {
                if entry.block_hash == block_hash {
                    entry.messages.insert(
                        validator_id.clone(),
                        ConsensusMessage::Vote {
                            block_hash,
                            vote_type: VoteType::Commit,
                            round: sequence,
                            view,
                            validator_id: validator_id.clone(),
                            signature,
                        },
                    );
                    entry.phase = PbftPhase::Commit;
                }
            }
        }

        // Check if we have enough COMMIT messages (2f+1)
        let commit_count = self.count_commit_messages(view, sequence, block_hash);
        let required_count = self.byzantine_threshold();

        if commit_count >= required_count {
            // Mark as committed
            {
                let mut committed = self.committed_certificates.write().unwrap();
                committed.insert((view, sequence, block_hash));
            }

            // Update phase to committed
            {
                let mut message_log = self.message_log.write().unwrap();
                if let Some(entry) = message_log.get_mut(&log_key) {
                    entry.phase = PbftPhase::Committed;
                }
            }

            // Advance sequence number
            {
                let mut current_sequence = self.current_sequence.write().unwrap();
                *current_sequence = sequence + 1;
            }

            tracing::info!("Block committed in PBFT: view={}, sequence={}", view, sequence);
        }

        Ok(vec![])
    }

    /// Handle VIEW-CHANGE message
    fn handle_view_change(
        &self,
        new_view: u64,
        validator_id: NodeId,
        signature: crate::types::Signature,
    ) -> Result<Vec<ConsensusMessage>> {
        // Verify validator is in validator set
        if !self.config.validator_set.contains(&validator_id) {
            return Ok(vec![]);
        }

        // Store view change vote
        {
            let mut view_change_votes = self.view_change_votes.write().unwrap();
            view_change_votes.entry(new_view).or_insert_with(HashSet::new).insert(validator_id);
        }

        // Check if we have enough view change votes
        let vote_count = {
            let view_change_votes = self.view_change_votes.read().unwrap();
            view_change_votes.get(&new_view).map(|votes| votes.len()).unwrap_or(0)
        };

        let required_count = self.byzantine_threshold();

        if vote_count >= required_count {
            // Trigger view change
            {
                let mut current_view = self.current_view.write().unwrap();
                *current_view = new_view;
            }

            // If we're the new primary, send NEW-VIEW message
            if self.is_primary(&self.config.node_id, new_view) {
                let new_view_message = ConsensusMessage::NewView {
                    view: new_view,
                    view_change_messages: vec![], // Simplified
                };

                return Ok(vec![new_view_message]);
            }
        }

        Ok(vec![])
    }

    /// Handle NEW-VIEW message
    fn handle_new_view(
        &self,
        view: u64,
        view_change_messages: Vec<ConsensusMessage>,
    ) -> Result<Vec<ConsensusMessage>> {
        // Verify the sender is the primary for the new view
        // This is simplified - in practice, we'd verify the NEW-VIEW message more thoroughly
        
        {
            let mut current_view = self.current_view.write().unwrap();
            *current_view = view;
        }

        // Clear view change votes for this view
        {
            let mut view_change_votes = self.view_change_votes.write().unwrap();
            view_change_votes.remove(&view);
        }

        tracing::info!("New view started: {}", view);

        Ok(vec![])
    }

    /// Check if a node is the primary for a given view
    fn is_primary(&self, node_id: &NodeId, view: u64) -> bool {
        if self.config.validator_set.is_empty() {
            return false;
        }
        
        let primary_index = (view as usize) % self.config.validator_set.len();
        &self.config.validator_set[primary_index] == node_id
    }

    /// Get the Byzantine fault threshold (2f+1)
    fn byzantine_threshold(&self) -> usize {
        (self.config.validator_set.len() * 2 / 3) + 1
    }

    /// Count PREPARE messages for a specific consensus instance
    fn count_prepare_messages(&self, view: u64, sequence: u64, block_hash: Hash) -> usize {
        let message_log = self.message_log.read().unwrap();
        let log_key = (view, sequence);
        
        if let Some(entry) = message_log.get(&log_key) {
            if entry.block_hash == block_hash {
                return entry.messages.values()
                    .filter(|msg| {
                        matches!(msg, ConsensusMessage::Vote { vote_type: VoteType::Prepare, .. })
                    })
                    .count();
            }
        }
        
        0
    }

    /// Count COMMIT messages for a specific consensus instance
    fn count_commit_messages(&self, view: u64, sequence: u64, block_hash: Hash) -> usize {
        let message_log = self.message_log.read().unwrap();
        let log_key = (view, sequence);
        
        if let Some(entry) = message_log.get(&log_key) {
            if entry.block_hash == block_hash {
                return entry.messages.values()
                    .filter(|msg| {
                        matches!(msg, ConsensusMessage::Vote { vote_type: VoteType::Commit, .. })
                    })
                    .count();
            }
        }
        
        0
    }

    /// Check if a block is prepared
    pub fn is_prepared(&self, view: u64, sequence: u64, block_hash: Hash) -> bool {
        let prepared = self.prepared_certificates.read().unwrap();
        prepared.contains(&(view, sequence, block_hash))
    }

    /// Check if a block is committed
    pub fn is_committed(&self, view: u64, sequence: u64, block_hash: Hash) -> bool {
        let committed = self.committed_certificates.read().unwrap();
        committed.contains(&(view, sequence, block_hash))
    }

    /// Get current view
    pub fn get_current_view(&self) -> u64 {
        *self.current_view.read().unwrap()
    }

    /// Get current sequence number
    pub fn get_current_sequence(&self) -> u64 {
        *self.current_sequence.read().unwrap()
    }

    /// Clean up old message logs (garbage collection)
    pub fn cleanup_old_logs(&self, keep_last_n: usize) {
        let current_sequence = *self.current_sequence.read().unwrap();
        
        if current_sequence <= keep_last_n as u64 {
            return;
        }

        let cutoff_sequence = current_sequence - keep_last_n as u64;
        
        {
            let mut message_log = self.message_log.write().unwrap();
            message_log.retain(|(_, sequence), _| *sequence >= cutoff_sequence);
        }

        {
            let mut prepared = self.prepared_certificates.write().unwrap();
            prepared.retain(|(_, sequence, _)| *sequence >= cutoff_sequence);
        }

        {
            let mut committed = self.committed_certificates.write().unwrap();
            committed.retain(|(_, sequence, _)| *sequence >= cutoff_sequence);
        }
    }
}
