use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

pub type Hash = [u8; 32];
pub type Address = [u8; 20];
pub type Signature = [u8; 64];
pub type PublicKey = [u8; 32];
pub type NodeId = String;
pub type BlockHeight = u64;
pub type Timestamp = u64;

/// Blockchain block structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub signatures: Vec<ValidatorSignature>,
}

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    pub height: BlockHeight,
    pub previous_hash: Hash,
    pub merkle_root: Hash,
    pub state_root: Hash,
    pub timestamp: Timestamp,
    pub proposer: NodeId,
    pub round: u64,
    pub view: u64,
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: Uuid,
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: Timestamp,
    pub signature: Signature,
    pub data: Vec<u8>,
}

/// Validator signature for consensus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatorSignature {
    pub validator_id: NodeId,
    pub signature: Signature,
    pub public_key: PublicKey,
}

/// Consensus message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    Propose {
        block: Block,
        round: u64,
        view: u64,
    },
    Vote {
        block_hash: Hash,
        vote_type: VoteType,
        round: u64,
        view: u64,
        validator_id: NodeId,
        signature: Signature,
    },
    ViewChange {
        new_view: u64,
        validator_id: NodeId,
        signature: Signature,
    },
    NewView {
        view: u64,
        view_change_messages: Vec<ConsensusMessage>,
    },
}

/// Vote types in BFT consensus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoteType {
    Prepare,
    Commit,
}

/// Network message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_id: Uuid,
    pub sender: NodeId,
    pub timestamp: Timestamp,
    pub payload: MessagePayload,
}

/// Different types of network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Consensus(ConsensusMessage),
    Transaction(Transaction),
    BlockRequest { height: BlockHeight },
    BlockResponse { block: Option<Block> },
    PeerDiscovery { peers: Vec<PeerInfo> },
    Heartbeat,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: NodeId,
    pub multiaddr: String,
    pub is_validator: bool,
    pub last_seen: Timestamp,
}

/// Node state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub current_height: BlockHeight,
    pub current_view: u64,
    pub current_round: u64,
    pub is_leader: bool,
    pub connected_peers: usize,
    pub mempool_size: usize,
}

impl Block {
    /// Calculate the hash of the block
    pub fn hash(&self) -> Hash {
        let serialized = bincode::serialize(self).expect("Failed to serialize block");
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }

    /// Verify block integrity
    pub fn verify(&self) -> bool {
        // Verify merkle root
        let calculated_merkle = self.calculate_merkle_root();
        if calculated_merkle != self.header.merkle_root {
            return false;
        }

        // Verify all transactions
        for tx in &self.transactions {
            if !tx.verify() {
                return false;
            }
        }

        true
    }

    /// Calculate merkle root of transactions
    pub fn calculate_merkle_root(&self) -> Hash {
        if self.transactions.is_empty() {
            return [0; 32];
        }

        let mut hashes: Vec<Hash> = self.transactions.iter().map(|tx| tx.hash()).collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // Duplicate if odd number
                }
                next_level.push(hasher.finalize().into());
            }
            hashes = next_level;
        }

        hashes[0]
    }
}

impl Transaction {
    /// Calculate the hash of the transaction
    pub fn hash(&self) -> Hash {
        let mut tx_for_hash = self.clone();
        tx_for_hash.signature = [0; 64]; // Zero out signature for hash calculation
        
        let serialized = bincode::serialize(&tx_for_hash).expect("Failed to serialize transaction");
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }

    /// Verify transaction signature (simplified)
    pub fn verify(&self) -> bool {
        // In a real implementation, this would verify the cryptographic signature
        // For now, we'll do basic validation
        self.amount > 0 && self.fee > 0 && self.from != self.to
    }

    /// Get transaction priority for mempool ordering
    pub fn priority(&self) -> u64 {
        self.fee // Simple fee-based priority
    }
}

impl NetworkMessage {
    pub fn new(sender: NodeId, payload: MessagePayload) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            sender,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            payload,
        }
    }
}
