use anyhow::Result;
use std::collections::HashMap;
use crate::types::{NetworkMessage, NodeId};

/// Gossip protocol handler for efficient message propagation
#[derive(Clone)]
pub struct GossipHandler {
    // Message cache to prevent loops
    seen_messages: HashMap<String, u64>,
    // Gossip parameters
    fanout: usize,
    gossip_factor: f64,
}

impl GossipHandler {
    pub fn new() -> Self {
        Self {
            seen_messages: HashMap::new(),
            fanout: 6, // Number of peers to gossip to
            gossip_factor: 0.25, // Fraction of peers to gossip to
        }
    }

    /// Handle incoming gossip message
    pub async fn handle_message(&mut self, message: NetworkMessage) -> Result<bool> {
        let message_id = format!("{:?}", message.message_id);
        
        // Check if we've seen this message before
        if self.seen_messages.contains_key(&message_id) {
            return Ok(false); // Already seen, don't propagate
        }

        // Mark as seen
        self.seen_messages.insert(message_id, message.timestamp);
        
        // Clean up old messages periodically
        self.cleanup_old_messages();
        
        Ok(true) // New message, should propagate
    }

    /// Clean up old seen messages
    fn cleanup_old_messages(&mut self) {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        // Remove messages older than 5 minutes
        self.seen_messages.retain(|_, &mut timestamp| {
            current_time - timestamp < 300_000
        });
    }
}
