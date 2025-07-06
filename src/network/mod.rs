use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, anyhow};
use tokio::sync::{mpsc, RwLock};
use libp2p::{
    gossipsub, identify, kad, mdns, noise, ping, yamux,
    core::upgrade,
    futures::StreamExt,
    identity, multiaddr,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, Multiaddr, PeerId, Swarm, Transport,
};
use tracing::{info, warn, error};

use crate::types::{NetworkMessage, MessagePayload, NodeId, PeerInfo};

pub mod gossip;
pub mod discovery;
pub mod transport;

use gossip::GossipHandler;
use discovery::DiscoveryHandler;

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub node_id: NodeId,
    pub listen_addresses: Vec<Multiaddr>,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub max_peers: usize,
    pub connection_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub gossip_heartbeat_interval: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            node_id: "default-node".to_string(),
            listen_addresses: vec!["/ip4/0.0.0.0/tcp/0".parse().unwrap()],
            bootstrap_peers: vec![],
            max_peers: 1000,
            connection_timeout: Duration::from_secs(10),
            heartbeat_interval: Duration::from_secs(30),
            gossip_heartbeat_interval: Duration::from_millis(700),
        }
    }
}

/// Network behavior combining all protocols
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "NetworkEvent")]
pub struct BlockchainBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}

/// Network events
#[derive(Debug)]
pub enum NetworkEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(kad::Event),
    Identify(identify::Event),
    Ping(ping::Event),
}

impl From<gossipsub::Event> for NetworkEvent {
    fn from(event: gossipsub::Event) -> Self {
        NetworkEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for NetworkEvent {
    fn from(event: mdns::Event) -> Self {
        NetworkEvent::Mdns(event)
    }
}

impl From<kad::Event> for NetworkEvent {
    fn from(event: kad::Event) -> Self {
        NetworkEvent::Kademlia(event)
    }
}

impl From<identify::Event> for NetworkEvent {
    fn from(event: identify::Event) -> Self {
        NetworkEvent::Identify(event)
    }
}

impl From<ping::Event> for NetworkEvent {
    fn from(event: ping::Event) -> Self {
        NetworkEvent::Ping(event)
    }
}

/// Network statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub gossip_messages_sent: u64,
    pub gossip_messages_received: u64,
}

/// Main network manager
pub struct NetworkManager {
    config: NetworkConfig,
    swarm: Swarm<BlockchainBehaviour>,
    
    // Message channels
    message_sender: mpsc::UnboundedSender<NetworkMessage>,
    message_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<NetworkMessage>>>>,
    
    // Outbound message queue
    outbound_sender: mpsc::UnboundedSender<(PeerId, NetworkMessage)>,
    outbound_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<(PeerId, NetworkMessage)>>>>,
    
    // Peer management
    connected_peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    peer_addresses: Arc<RwLock<HashMap<PeerId, Multiaddr>>>,
    
    // Protocol handlers
    gossip_handler: GossipHandler,
    discovery_handler: DiscoveryHandler,
    
    // Statistics
    stats: Arc<RwLock<NetworkStats>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        // Generate or load identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        info!("Local peer id: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(config.gossip_heartbeat_interval)
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|message| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                message.data.hash(&mut hasher);
                gossipsub::MessageId::from(hasher.finish().to_string())
            })
            .build()
            .map_err(|e| anyhow!("Failed to create gossipsub config: {}", e))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Subscribe to blockchain topics
        let block_topic = gossipsub::IdentTopic::new("blockchain/blocks");
        let tx_topic = gossipsub::IdentTopic::new("blockchain/transactions");
        let consensus_topic = gossipsub::IdentTopic::new("blockchain/consensus");
        
        gossipsub.subscribe(&block_topic)?;
        gossipsub.subscribe(&tx_topic)?;
        gossipsub.subscribe(&consensus_topic)?;

        // Create mDNS for local discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Create Kademlia for DHT
        let store = kad::store::MemoryStore::new(local_peer_id);
        let mut kademlia = kad::Behaviour::new(local_peer_id, store);
        
        // Add bootstrap nodes to Kademlia
        for addr in &config.bootstrap_peers {
            if let Some(peer_id) = extract_peer_id(addr) {
                kademlia.add_address(&peer_id, addr.clone());
            }
        }

        // Create identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/blockchain-node/1.0.0".to_string(),
            local_key.public(),
        ));

        // Create ping protocol
        let ping = ping::Behaviour::new(ping::Config::new());

        // Create network behavior
        let behaviour = BlockchainBehaviour {
            gossipsub,
            mdns,
            kademlia,
            identify,
            ping,
        };

        // Create swarm
        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, local_peer_id);

        // Listen on configured addresses
        for addr in &config.listen_addresses {
            swarm.listen_on(addr.clone())?;
        }

        // Create message channels
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        let (outbound_sender, outbound_receiver) = mpsc::unbounded_channel();

        // Create protocol handlers
        let gossip_handler = GossipHandler::new();
        let discovery_handler = DiscoveryHandler::new();

        Ok(Self {
            config,
            swarm,
            message_sender,
            message_receiver: Arc::new(RwLock::new(Some(message_receiver))),
            outbound_sender,
            outbound_receiver: Arc::new(RwLock::new(Some(outbound_receiver))),
            connected_peers: Arc::new(RwLock::new(HashMap::new())),
            peer_addresses: Arc::new(RwLock::new(HashMap::new())),
            gossip_handler,
            discovery_handler,
            stats: Arc::new(RwLock::new(NetworkStats::default())),
        })
    }

    /// Start the network manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting network manager...");

        // Connect to bootstrap peers
        for addr in &self.config.bootstrap_peers.clone() {
            if let Err(e) = self.swarm.dial(addr.clone()) {
                warn!("Failed to dial bootstrap peer {}: {}", addr, e);
            }
        }

        // Start Kademlia bootstrap
        if let Err(e) = self.swarm.behaviour_mut().kademlia.bootstrap() {
            warn!("Failed to start Kademlia bootstrap: {}", e);
        }

        // Take message receivers
        let message_receiver = {
            let mut receiver_guard = self.message_receiver.write().await;
            receiver_guard.take().ok_or_else(|| anyhow!("Network already started"))?
        };

        let outbound_receiver = {
            let mut receiver_guard = self.outbound_receiver.write().await;
            receiver_guard.take().ok_or_else(|| anyhow!("Network already started"))?
        };

        // Spawn message processing tasks
        let network_manager = self.clone_for_tasks().await;
        tokio::spawn(async move {
            network_manager.process_inbound_messages(message_receiver).await;
        });

        let network_manager = self.clone_for_tasks().await;
        tokio::spawn(async move {
            network_manager.process_outbound_messages(outbound_receiver).await;
        });

        // Start main event loop
        self.run_event_loop().await
    }

    /// Main event loop for processing network events
    async fn run_event_loop(&mut self) -> Result<()> {
        let mut heartbeat_interval = tokio::time::interval(self.config.heartbeat_interval);

        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    if let Err(e) = self.handle_swarm_event(event).await {
                        error!("Error handling swarm event: {}", e);
                    }
                }
                _ = heartbeat_interval.tick() => {
                    self.send_heartbeat().await;
                }
            }
        }
    }

    /// Handle swarm events
    async fn handle_swarm_event(&mut self, event: SwarmEvent<NetworkEvent>) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                info!("Connected to peer: {}", peer_id);
                
                // Store peer information
                let peer_info = PeerInfo {
                    node_id: peer_id.to_string(),
                    multiaddr: endpoint.get_remote_address().to_string(),
                    is_validator: false, // Will be determined later
                    last_seen: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                };

                {
                    let mut connected_peers = self.connected_peers.write().await;
                    connected_peers.insert(peer_id, peer_info);
                }

                {
                    let mut peer_addresses = self.peer_addresses.write().await;
                    peer_addresses.insert(peer_id, endpoint.get_remote_address().clone());
                }

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.connected_peers = self.connected_peers.read().await.len();
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                
                {
                    let mut connected_peers = self.connected_peers.write().await;
                    connected_peers.remove(&peer_id);
                }

                {
                    let mut peer_addresses = self.peer_addresses.write().await;
                    peer_addresses.remove(&peer_id);
                }

                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.connected_peers = self.connected_peers.read().await.len();
                }
            }
            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event).await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle behavior-specific events
    async fn handle_behaviour_event(&mut self, event: NetworkEvent) -> Result<()> {
        match event {
            NetworkEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message_id: _,
                message,
            }) => {
                self.handle_gossip_message(propagation_source, message).await?;
            }
            NetworkEvent::Mdns(mdns::Event::Discovered(list)) => {
                for (peer_id, multiaddr) in list {
                    info!("Discovered peer via mDNS: {} at {}", peer_id, multiaddr);
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                }
            }
            NetworkEvent::Kademlia(kad::Event::OutboundQueryProgressed { result, .. }) => {
                self.discovery_handler.handle_kademlia_result(result).await?;
            }
            NetworkEvent::Identify(identify::Event::Received { peer_id, info }) => {
                info!("Identified peer {}: {}", peer_id, info.protocol_version);
                
                // Add addresses to Kademlia
                for addr in info.listen_addrs {
                    self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            NetworkEvent::Ping(ping::Event { peer, result }) => {
                match result {
                    Ok(duration) => {
                        // Update peer last seen time
                        if let Ok(mut connected_peers) = self.connected_peers.try_write() {
                            if let Some(peer_info) = connected_peers.get_mut(&peer) {
                                peer_info.last_seen = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Ping failed for peer {}: {}", peer, e);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle incoming gossip messages
    async fn handle_gossip_message(
        &mut self,
        source: PeerId,
        message: gossipsub::Message,
    ) -> Result<()> {
        // Deserialize network message
        let network_message: NetworkMessage = bincode::deserialize(&message.data)
            .map_err(|e| anyhow!("Failed to deserialize message: {}", e))?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_messages_received += 1;
            stats.total_bytes_received += message.data.len() as u64;
            stats.gossip_messages_received += 1;
        }

        // Forward to message processing
        if let Err(e) = self.message_sender.send(network_message) {
            error!("Failed to forward message: {}", e);
        }

        Ok(())
    }

    /// Process inbound messages
    async fn process_inbound_messages(&self, mut receiver: mpsc::UnboundedReceiver<NetworkMessage>) {
        while let Some(message) = receiver.recv().await {
            // Process message based on payload type
            match &message.payload {
                MessagePayload::Consensus(_) => {
                    // Forward to consensus engine
                    info!("Received consensus message from {}", message.sender);
                }
                MessagePayload::Transaction(_) => {
                    // Forward to mempool
                    info!("Received transaction from {}", message.sender);
                }
                MessagePayload::BlockRequest { .. } => {
                    // Handle block request
                    info!("Received block request from {}", message.sender);
                }
                MessagePayload::BlockResponse { .. } => {
                    // Handle block response
                    info!("Received block response from {}", message.sender);
                }
                MessagePayload::PeerDiscovery { .. } => {
                    // Handle peer discovery
                    info!("Received peer discovery from {}", message.sender);
                }
                MessagePayload::Heartbeat => {
                    // Handle heartbeat
                    // Update peer last seen time
                }
            }
        }
    }

    /// Process outbound messages
    async fn process_outbound_messages(&self, mut receiver: mpsc::UnboundedReceiver<(PeerId, NetworkMessage)>) {
        while let Some((peer_id, message)) = receiver.recv().await {
            if let Err(e) = self.send_message_to_peer(peer_id, message).await {
                error!("Failed to send message to peer {}: {}", peer_id, e);
            }
        }
    }

    /// Send message to specific peer
    async fn send_message_to_peer(&self, peer_id: PeerId, message: NetworkMessage) -> Result<()> {
        // Serialize message
        let data = bincode::serialize(&message)
            .map_err(|e| anyhow!("Failed to serialize message: {}", e))?;

        // Determine topic based on message type
        let topic = match &message.payload {
            MessagePayload::Consensus(_) => gossipsub::IdentTopic::new("blockchain/consensus"),
            MessagePayload::Transaction(_) => gossipsub::IdentTopic::new("blockchain/transactions"),
            MessagePayload::BlockRequest { .. } | MessagePayload::BlockResponse { .. } => {
                gossipsub::IdentTopic::new("blockchain/blocks")
            }
            _ => gossipsub::IdentTopic::new("blockchain/general"),
        };

        // This is a simplified implementation - in practice, you'd want more sophisticated routing
        // For now, we'll just publish to the topic
        Ok(())
    }

    /// Broadcast message to all peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<()> {
        // Serialize message
        let data = bincode::serialize(&message)
            .map_err(|e| anyhow!("Failed to serialize message: {}", e))?;

        // Determine topic
        let topic = match &message.payload {
            MessagePayload::Consensus(_) => gossipsub::IdentTopic::new("blockchain/consensus"),
            MessagePayload::Transaction(_) => gossipsub::IdentTopic::new("blockchain/transactions"),
            MessagePayload::BlockRequest { .. } | MessagePayload::BlockResponse { .. } => {
                gossipsub::IdentTopic::new("blockchain/blocks")
            }
            _ => gossipsub::IdentTopic::new("blockchain/general"),
        };

        // This would need access to the swarm, which requires refactoring
        // For now, this is a placeholder
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_messages_sent += 1;
            stats.total_bytes_sent += data.len() as u64;
            stats.gossip_messages_sent += 1;
        }

        Ok(())
    }

    /// Send heartbeat to all connected peers
    async fn send_heartbeat(&self) {
        let heartbeat_message = NetworkMessage::new(
            self.config.node_id.clone(),
            MessagePayload::Heartbeat,
        );

        if let Err(e) = self.broadcast_message(heartbeat_message).await {
            error!("Failed to send heartbeat: {}", e);
        }
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        let connected_peers = self.connected_peers.read().await;
        connected_peers.values().cloned().collect()
    }

    /// Get message sender for other components
    pub fn get_message_sender(&self) -> mpsc::UnboundedSender<NetworkMessage> {
        self.message_sender.clone()
    }

    /// Clone for async tasks (simplified)
    async fn clone_for_tasks(&self) -> Self {
        // This is a simplified clone for demonstration
        // In practice, you'd need to carefully handle the shared state
        Self {
            config: self.config.clone(),
            swarm: self.swarm.clone(), // This won't work in practice
            message_sender: self.message_sender.clone(),
            message_receiver: self.message_receiver.clone(),
            outbound_sender: self.outbound_sender.clone(),
            outbound_receiver: self.outbound_receiver.clone(),
            connected_peers: self.connected_peers.clone(),
            peer_addresses: self.peer_addresses.clone(),
            gossip_handler: self.gossip_handler.clone(),
            discovery_handler: self.discovery_handler.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Extract peer ID from multiaddress
fn extract_peer_id(addr: &Multiaddr) -> Option<PeerId> {
    for protocol in addr.iter() {
        if let multiaddr::Protocol::P2p(peer_id) = protocol {
            return Some(peer_id);
        }
    }
    None
}
