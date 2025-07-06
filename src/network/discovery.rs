use anyhow::Result;
use libp2p::kad;
use std::collections::HashSet;
use crate::types::{NodeId, PeerInfo};

/// Peer discovery handler
#[derive(Clone)]
pub struct DiscoveryHandler {
    discovered_peers: HashSet<NodeId>,
}

impl DiscoveryHandler {
    pub fn new() -> Self {
        Self {
            discovered_peers: HashSet::new(),
        }
    }

    /// Handle Kademlia query results
    pub async fn handle_kademlia_result(&mut self, result: kad::QueryResult) -> Result<()> {
        match result {
            kad::QueryResult::Bootstrap(Ok(kad::BootstrapOk { peer, .. })) => {
                self.discovered_peers.insert(peer.to_string());
            }
            kad::QueryResult::GetClosestPeers(Ok(kad::GetClosestPeersOk { peers, .. })) => {
                for peer in peers {
                    self.discovered_peers.insert(peer.to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Get discovered peers
    pub fn get_discovered_peers(&self) -> Vec<NodeId> {
        self.discovered_peers.iter().cloned().collect()
    }
}
