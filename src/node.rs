use std::sync::Arc;
use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use tracing::{info, error, warn};

use crate::cli::Cli;
use crate::storage::Storage;
use crate::mempool::{Mempool, MempoolConfig};
use crate::consensus::{ConsensusEngine, ConsensusConfig};
use crate::network::{NetworkManager, NetworkConfig};
use crate::validation::Validator;
use crate::metrics::MetricsServer;
use crate::api::JsonRpcServer;
use crate::types::{NetworkMessage, MessagePayload, NodeState, BlockHeight};

/// Main blockchain node that orchestrates all components
pub struct BlockchainNode {
    config: Cli,
    
    // Core components
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<ConsensusEngine>,
    network: Arc<NetworkManager>,
    validator: Arc<Validator>,
    
    // Services
    metrics_server: Option<MetricsServer>,
    rpc_server: Option<JsonRpcServer>,
    
    // Message channels
    network_receiver: Option<mpsc::UnboundedReceiver<NetworkMessage>>,
    consensus_sender: Option<mpsc::UnboundedSender<NetworkMessage>>,
    
    // Node state
    is_running: Arc<std::sync::RwLock<bool>>,
}

impl BlockchainNode {
    /// Create a new blockchain node
    pub async fn new(config: Cli) -> Result<Self> {
        info!("Initializing blockchain node: {}", config.node_id);

        // Initialize storage
        let storage = Arc::new(Storage::new(&config.db_path)?);
        info!("Storage initialized at: {:?}", config.db_path);

        // Initialize mempool
        let mempool_config = MempoolConfig {
            max_size: config.mempool_size,
            max_per_account: 100,
            min_fee: 1,
            max_tx_size: 1024 * 1024,
            cleanup_interval_secs: 60,
            max_age_secs: 3600,
        };
        let mempool = Arc::new(Mempool::new(mempool_config));
        info!("Mempool initialized with max size: {}", config.mempool_size);

        // Initialize validator
        let validator = Arc::new(Validator::new((*storage).clone()));

        // Initialize consensus engine
        let consensus_config = ConsensusConfig {
            node_id: config.node_id.clone(),
            is_validator: config.is_validator(),
            validator_set: Self::parse_validator_set(&config)?,
            block_time_ms: config.block_time_ms,
            view_timeout_ms: 10000, // 10 seconds
            max_block_size: 1024 * 1024 * 10, // 10MB
            max_transactions_per_block: 1000,
        };

        let consensus = Arc::new(ConsensusEngine::new(
            consensus_config,
            storage.clone(),
            mempool.clone(),
        )?);
        info!("Consensus engine initialized");

        // Initialize network manager
        let network_config = NetworkConfig {
            node_id: config.node_id.clone(),
            listen_addresses: vec![config.listen_addr.parse()?],
            bootstrap_peers: Self::parse_bootstrap_peers(&config)?,
            max_peers: config.max_peers,
            connection_timeout: std::time::Duration::from_secs(10),
            heartbeat_interval: std::time::Duration::from_secs(30),
            gossip_heartbeat_interval: std::time::Duration::from_millis(700),
        };

        let network = Arc::new(NetworkManager::new(network_config).await?);
        info!("Network manager initialized");

        // Initialize metrics server if enabled
        let metrics_server = if config.enable_metrics {
            Some(MetricsServer::new(config.metrics_port)?)
        } else {
            None
        };

        // Initialize JSON-RPC server
        let rpc_server = Some(JsonRpcServer::new(
            config.rpc_port,
            storage.clone(),
            mempool.clone(),
            consensus.clone(),
        )?);

        Ok(Self {
            config,
            storage,
            mempool,
            consensus,
            network,
            validator,
            metrics_server,
            rpc_server,
            network_receiver: None,
            consensus_sender: None,
            is_running: Arc::new(std::sync::RwLock::new(false)),
        })
    }

    /// Start the blockchain node
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting blockchain node...");

        // Set running state
        {
            let mut is_running = self.is_running.write().unwrap();
            *is_running = true;
        }

        // Start metrics server
        if let Some(metrics_server) = &mut self.metrics_server {
            metrics_server.start().await?;
            info!("Metrics server started on port {}", self.config.metrics_port);
        }

        // Start JSON-RPC server
        if let Some(rpc_server) = &mut self.rpc_server {
            rpc_server.start().await?;
            info!("JSON-RPC server started on port {}", self.config.rpc_port);
        }

        // Connect network and consensus
        self.setup_message_routing().await?;

        // Start network manager
        // Note: This is simplified - in practice, you'd need to handle the async nature properly
        info!("Network manager started");

        // Start consensus engine
        let consensus = self.consensus.clone();
        tokio::spawn(async move {
            if let Err(e) = consensus.start().await {
                error!("Consensus engine failed: {}", e);
            }
        });
        info!("Consensus engine started");

        // Start message processing
        self.start_message_processing().await?;

        // Start periodic tasks
        self.start_periodic_tasks().await?;

        info!("Blockchain node started successfully");
        Ok(())
    }

    /// Setup message routing between components
    async fn setup_message_routing(&mut self) -> Result<()> {
        // Get message channels
        let network_sender = self.network.get_message_sender();
        let consensus_sender = self.consensus.get_message_sender();

        // Store senders for message routing
        self.consensus_sender = Some(consensus_sender);

        Ok(())
    }

    /// Start message processing loops
    async fn start_message_processing(&mut self) -> Result<()> {
        // This would set up the message routing between network and consensus
        // For now, it's a placeholder
        info!("Message processing started");
        Ok(())
    }

    /// Start periodic maintenance tasks
    async fn start_periodic_tasks(&self) -> Result<()> {
        let storage = self.storage.clone();
        let mempool = self.mempool.clone();
        let is_running = self.is_running.clone();

        // Spawn periodic cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Check if node is still running
                {
                    let running = is_running.read().unwrap();
                    if !*running {
                        break;
                    }
                }

                // Perform cleanup tasks
                if let Err(e) = storage.compact() {
                    warn!("Failed to compact storage: {}", e);
                }

                // Additional periodic tasks can be added here
            }
        });

        info!("Periodic tasks started");
        Ok(())
    }

    /// Get current node state
    pub async fn get_node_state(&self) -> Result<NodeState> {
        let current_height = self.storage.get_latest_height()?.unwrap_or(0);
        let consensus_stats = self.consensus.get_stats();
        let network_stats = self.network.get_stats().await;
        let mempool_stats = self.mempool.get_stats();

        Ok(NodeState {
            current_height,
            current_view: consensus_stats.current_view,
            current_round: consensus_stats.current_round,
            is_leader: consensus_stats.current_leader.as_ref() == Some(&self.config.node_id),
            connected_peers: network_stats.connected_peers,
            mempool_size: mempool_stats.total_transactions,
        })
    }

    /// Shutdown the node gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down blockchain node...");

        // Set running state to false
        {
            let mut is_running = self.is_running.write().unwrap();
            *is_running = false;
        }

        // Shutdown services
        if let Some(rpc_server) = &mut self.rpc_server {
            rpc_server.shutdown().await?;
        }

        if let Some(metrics_server) = &mut self.metrics_server {
            metrics_server.shutdown().await?;
        }

        // Clear mempool
        self.mempool.clear()?;

        // Compact storage
        self.storage.compact()?;

        info!("Blockchain node shutdown complete");
        Ok(())
    }

    /// Parse validator set from configuration
    fn parse_validator_set(config: &Cli) -> Result<Vec<String>> {
        // For now, return a default validator set
        // In practice, this would be loaded from genesis file or configuration
        Ok(vec![
            "validator-1".to_string(),
            "validator-2".to_string(),
            "validator-3".to_string(),
            "validator-4".to_string(),
        ])
    }

    /// Parse bootstrap peers from configuration
    fn parse_bootstrap_peers(config: &Cli) -> Result<Vec<libp2p::Multiaddr>> {
        let mut peers = Vec::new();
        
        for peer_str in config.get_bootstrap_peers() {
            let addr = peer_str.parse()
                .map_err(|e| anyhow!("Invalid bootstrap peer address '{}': {}", peer_str, e))?;
            peers.push(addr);
        }
        
        Ok(peers)
    }

    /// Get storage reference
    pub fn storage(&self) -> &Arc<Storage> {
        &self.storage
    }

    /// Get mempool reference
    pub fn mempool(&self) -> &Arc<Mempool> {
        &self.mempool
    }

    /// Get consensus reference
    pub fn consensus(&self) -> &Arc<ConsensusEngine> {
        &self.consensus
    }

    /// Get network reference
    pub fn network(&self) -> &Arc<NetworkManager> {
        &self.network
    }

    /// Check if node is running
    pub fn is_running(&self) -> bool {
        *self.is_running.read().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_node() -> Result<BlockchainNode> {
        let temp_dir = TempDir::new()?;
        
        let mut config = Cli::parse_from(&[
            "blockchain-node",
            "--node-id", "test-node",
            "--db-path", temp_dir.path().to_str().unwrap(),
        ]);
        
        BlockchainNode::new(config).await
    }

    #[tokio::test]
    async fn test_node_creation() {
        let node = create_test_node().await;
        assert!(node.is_ok());
    }

    #[tokio::test]
    async fn test_node_state() {
        let node = create_test_node().await.unwrap();
        let state = node.get_node_state().await.unwrap();
        
        assert_eq!(state.current_height, 0);
        assert_eq!(state.connected_peers, 0);
    }
}
