// Transport layer utilities for the blockchain network
// This module provides helper functions for network transport configuration

use anyhow::Result;
use libp2p::{Multiaddr, PeerId};

/// Parse a multiaddress string
pub fn parse_multiaddr(addr: &str) -> Result<Multiaddr> {
    addr.parse().map_err(|e| anyhow::anyhow!("Invalid multiaddress: {}", e))
}

/// Extract peer ID from multiaddress
pub fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    for protocol in addr.iter() {
        if let libp2p::multiaddr::Protocol::P2p(peer_id) = protocol {
            return Some(peer_id);
        }
    }
    None
}

/// Validate multiaddress format
pub fn validate_multiaddr(addr: &Multiaddr) -> bool {
    // Basic validation - check if it has required components
    let mut has_transport = false;
    let mut has_peer_id = false;
    
    for protocol in addr.iter() {
        match protocol {
            libp2p::multiaddr::Protocol::Tcp(_) | 
            libp2p::multiaddr::Protocol::Udp(_) => {
                has_transport = true;
            }
            libp2p::multiaddr::Protocol::P2p(_) => {
                has_peer_id = true;
            }
            _ => {}
        }
    }
    
    has_transport // Peer ID is optional for some use cases
}
