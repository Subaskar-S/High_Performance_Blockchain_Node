use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "blockchain-node")]
#[command(about = "High-throughput, fault-tolerant blockchain node")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// Node operation mode
    #[arg(short, long, default_value = "validator")]
    pub mode: NodeMode,

    /// Node identifier
    #[arg(long, default_value = "node-1")]
    pub node_id: String,

    /// Listen address for P2P networking
    #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
    pub listen_addr: String,

    /// Bootstrap peers (comma-separated multiaddresses)
    #[arg(long)]
    pub bootstrap_peers: Option<String>,

    /// Path to genesis file
    #[arg(long, default_value = "config/genesis.json")]
    pub genesis_file: PathBuf,

    /// Path to node configuration file
    #[arg(long)]
    pub config_file: Option<PathBuf>,

    /// Database path
    #[arg(long, default_value = "./data")]
    pub db_path: PathBuf,

    /// JSON-RPC server port
    #[arg(long, default_value = "8545")]
    pub rpc_port: u16,

    /// Metrics server port
    #[arg(long, default_value = "9090")]
    pub metrics_port: u16,

    /// Enable metrics collection
    #[arg(long, default_value = "true")]
    pub enable_metrics: bool,

    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Maximum number of peers
    #[arg(long, default_value = "1000")]
    pub max_peers: usize,

    /// Block time in milliseconds
    #[arg(long, default_value = "1000")]
    pub block_time_ms: u64,

    /// Transaction pool size limit
    #[arg(long, default_value = "10000")]
    pub mempool_size: usize,

    /// Enable development mode (faster consensus, less security)
    #[arg(long, default_value = "false")]
    pub dev_mode: bool,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum NodeMode {
    /// Full validator node participating in consensus
    Validator,
    /// Observer node that syncs but doesn't participate in consensus
    Observer,
    /// Archive node that stores full history
    Archive,
}

impl Cli {
    /// Parse bootstrap peers from comma-separated string
    pub fn get_bootstrap_peers(&self) -> Vec<String> {
        self.bootstrap_peers
            .as_ref()
            .map(|peers| {
                peers
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if node should participate in consensus
    pub fn is_validator(&self) -> bool {
        matches!(self.mode, NodeMode::Validator)
    }

    /// Check if node should store full history
    pub fn is_archive(&self) -> bool {
        matches!(self.mode, NodeMode::Archive)
    }
}
