use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::storage::Storage;
use crate::mempool::Mempool;
use crate::consensus::ConsensusEngine;
use crate::types::{Block, Transaction, BlockHeight, Hash, NodeState};

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Block information for API responses
#[derive(Debug, Serialize)]
pub struct BlockInfo {
    pub height: BlockHeight,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub proposer: String,
    pub transaction_count: usize,
    pub size: usize,
}

/// Transaction information for API responses
#[derive(Debug, Serialize)]
pub struct TransactionInfo {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub status: String,
}

/// Node status information
#[derive(Debug, Serialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub is_validator: bool,
    pub current_height: BlockHeight,
    pub current_view: u64,
    pub current_round: u64,
    pub connected_peers: usize,
    pub mempool_size: usize,
    pub is_syncing: bool,
}

/// JSON-RPC server for blockchain API
pub struct JsonRpcServer {
    port: u16,
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<ConsensusEngine>,
    is_running: Arc<RwLock<bool>>,
}

impl JsonRpcServer {
    /// Create new JSON-RPC server
    pub fn new(
        port: u16,
        storage: Arc<Storage>,
        mempool: Arc<Mempool>,
        consensus: Arc<ConsensusEngine>,
    ) -> Result<Self> {
        Ok(Self {
            port,
            storage,
            mempool,
            consensus,
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the JSON-RPC server
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }

        let storage = self.storage.clone();
        let mempool = self.mempool.clone();
        let consensus = self.consensus.clone();
        let is_running = self.is_running.clone();
        let port = self.port;

        tokio::spawn(async move {
            // This is a simplified HTTP server implementation
            // In practice, you'd use jsonrpsee or similar library
            
            let handler = JsonRpcHandler::new(storage, mempool, consensus);
            
            // Placeholder for actual HTTP server
            tracing::info!("JSON-RPC server would start on port {}", port);
            
            // Keep running until shutdown
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                
                let running = is_running.read().await;
                if !*running {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Shutdown the JSON-RPC server
    pub async fn shutdown(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }
}

/// JSON-RPC method handler
pub struct JsonRpcHandler {
    storage: Arc<Storage>,
    mempool: Arc<Mempool>,
    consensus: Arc<ConsensusEngine>,
}

impl JsonRpcHandler {
    pub fn new(
        storage: Arc<Storage>,
        mempool: Arc<Mempool>,
        consensus: Arc<ConsensusEngine>,
    ) -> Self {
        Self {
            storage,
            mempool,
            consensus,
        }
    }

    /// Handle JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "blockchain_getBlockByHeight" => self.get_block_by_height(request.params).await,
            "blockchain_getBlockByHash" => self.get_block_by_hash(request.params).await,
            "blockchain_getLatestBlock" => self.get_latest_block().await,
            "blockchain_getTransaction" => self.get_transaction(request.params).await,
            "blockchain_sendTransaction" => self.send_transaction(request.params).await,
            "blockchain_getBalance" => self.get_balance(request.params).await,
            "blockchain_getNodeStatus" => self.get_node_status().await,
            "blockchain_getPeers" => self.get_peers().await,
            "blockchain_getMempoolInfo" => self.get_mempool_info().await,
            _ => Err(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: request.id,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id: request.id,
            },
        }
    }

    /// Get block by height
    async fn get_block_by_height(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        let height: BlockHeight = params
            .and_then(|p| p.as_u64())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Invalid params: height required".to_string(),
                data: None,
            })?;

        match self.storage.blocks().get_block(height) {
            Ok(Some(block)) => {
                let block_info = self.block_to_info(&block);
                serde_json::to_value(block_info).map_err(|e| JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {}", e),
                    data: None,
                })
            }
            Ok(None) => Err(JsonRpcError {
                code: -32000,
                message: "Block not found".to_string(),
                data: None,
            }),
            Err(e) => Err(JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        }
    }

    /// Get block by hash
    async fn get_block_by_hash(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        let hash_str: String = params
            .and_then(|p| p.as_str().map(|s| s.to_string()))
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Invalid params: hash required".to_string(),
                data: None,
            })?;

        // Parse hash from hex string (simplified)
        let hash = self.parse_hash(&hash_str)?;

        match self.storage.blocks().get_block_by_hash(&hash) {
            Ok(Some(block)) => {
                let block_info = self.block_to_info(&block);
                serde_json::to_value(block_info).map_err(|e| JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {}", e),
                    data: None,
                })
            }
            Ok(None) => Err(JsonRpcError {
                code: -32000,
                message: "Block not found".to_string(),
                data: None,
            }),
            Err(e) => Err(JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        }
    }

    /// Get latest block
    async fn get_latest_block(&self) -> Result<serde_json::Value, JsonRpcError> {
        match self.storage.blocks().get_latest_block() {
            Ok(Some(block)) => {
                let block_info = self.block_to_info(&block);
                serde_json::to_value(block_info).map_err(|e| JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {}", e),
                    data: None,
                })
            }
            Ok(None) => Err(JsonRpcError {
                code: -32000,
                message: "No blocks found".to_string(),
                data: None,
            }),
            Err(e) => Err(JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        }
    }

    /// Get transaction
    async fn get_transaction(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        let tx_id_str: String = params
            .and_then(|p| p.as_str().map(|s| s.to_string()))
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Invalid params: transaction ID required".to_string(),
                data: None,
            })?;

        let tx_id = Uuid::parse_str(&tx_id_str).map_err(|_| JsonRpcError {
            code: -32602,
            message: "Invalid transaction ID format".to_string(),
            data: None,
        })?;

        match self.storage.transactions().get_transaction(&tx_id) {
            Ok(Some(tx)) => {
                let tx_info = self.transaction_to_info(&tx, "confirmed");
                serde_json::to_value(tx_info).map_err(|e| JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {}", e),
                    data: None,
                })
            }
            Ok(None) => {
                // Check mempool
                if let Some(tx) = self.mempool.get_transaction(&tx_id) {
                    let tx_info = self.transaction_to_info(&tx, "pending");
                    serde_json::to_value(tx_info).map_err(|e| JsonRpcError {
                        code: -32603,
                        message: format!("Internal error: {}", e),
                        data: None,
                    })
                } else {
                    Err(JsonRpcError {
                        code: -32000,
                        message: "Transaction not found".to_string(),
                        data: None,
                    })
                }
            }
            Err(e) => Err(JsonRpcError {
                code: -32603,
                message: format!("Internal error: {}", e),
                data: None,
            }),
        }
    }

    /// Send transaction
    async fn send_transaction(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        // This would parse transaction from params and add to mempool
        // Simplified implementation
        Err(JsonRpcError {
            code: -32601,
            message: "Method not implemented".to_string(),
            data: None,
        })
    }

    /// Get balance
    async fn get_balance(&self, params: Option<serde_json::Value>) -> Result<serde_json::Value, JsonRpcError> {
        // This would get account balance from state
        // Simplified implementation
        serde_json::to_value(0u64).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Internal error: {}", e),
            data: None,
        })
    }

    /// Get node status
    async fn get_node_status(&self) -> Result<serde_json::Value, JsonRpcError> {
        let stats = self.consensus.get_stats();
        let mempool_stats = self.mempool.get_stats();

        let status = NodeStatus {
            node_id: "blockchain-node".to_string(), // Would get from config
            is_validator: true, // Would get from config
            current_height: stats.current_height,
            current_view: stats.current_view,
            current_round: stats.current_round,
            connected_peers: 0, // Would get from network
            mempool_size: mempool_stats.total_transactions,
            is_syncing: false, // Would determine from sync status
        };

        serde_json::to_value(status).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Internal error: {}", e),
            data: None,
        })
    }

    /// Get peers
    async fn get_peers(&self) -> Result<serde_json::Value, JsonRpcError> {
        // Would get from network manager
        serde_json::to_value(Vec::<String>::new()).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Internal error: {}", e),
            data: None,
        })
    }

    /// Get mempool info
    async fn get_mempool_info(&self) -> Result<serde_json::Value, JsonRpcError> {
        let stats = self.mempool.get_stats();
        serde_json::to_value(stats).map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Internal error: {}", e),
            data: None,
        })
    }

    /// Convert block to API info
    fn block_to_info(&self, block: &Block) -> BlockInfo {
        BlockInfo {
            height: block.header.height,
            hash: format!("{:x?}", block.hash()),
            previous_hash: format!("{:x?}", block.header.previous_hash),
            timestamp: block.header.timestamp,
            proposer: block.header.proposer.clone(),
            transaction_count: block.transactions.len(),
            size: bincode::serialize(block).unwrap_or_default().len(),
        }
    }

    /// Convert transaction to API info
    fn transaction_to_info(&self, tx: &Transaction, status: &str) -> TransactionInfo {
        TransactionInfo {
            id: tx.id.to_string(),
            from: format!("{:x?}", tx.from),
            to: format!("{:x?}", tx.to),
            amount: tx.amount,
            fee: tx.fee,
            nonce: tx.nonce,
            timestamp: tx.timestamp,
            status: status.to_string(),
        }
    }

    /// Parse hash from hex string (simplified)
    fn parse_hash(&self, hash_str: &str) -> Result<Hash, JsonRpcError> {
        // Simplified hash parsing
        Ok([0; 32]) // Placeholder
    }
}
