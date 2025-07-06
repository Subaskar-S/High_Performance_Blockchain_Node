use clap::Parser;
use tracing::{info, error};

mod cli;
mod consensus;
mod network;
mod storage;
mod api;
mod metrics;
mod types;
mod mempool;
mod validation;
mod node;

use cli::Cli;
use node::BlockchainNode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Parse command line arguments
    let cli = Cli::parse();
    
    info!("Starting blockchain node in {:?} mode", cli.mode);
    info!("Node ID: {}", cli.node_id);
    info!("Listen address: {}", cli.listen_addr);

    // Create and start the blockchain node
    let mut node = BlockchainNode::new(cli).await?;
    
    // Start the node
    if let Err(e) = node.start().await {
        error!("Failed to start node: {}", e);
        return Err(e);
    }

    // Keep the node running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down blockchain node...");
    
    node.shutdown().await?;
    info!("Node shutdown complete");

    Ok(())
}
