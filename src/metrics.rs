use std::sync::Arc;
use anyhow::{Result, anyhow};
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, 
    Registry, Encoder, TextEncoder
};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Blockchain node metrics collector
pub struct NodeMetrics {
    // Block metrics
    pub blocks_processed: IntCounter,
    pub block_height: IntGauge,
    pub block_processing_time: Histogram,
    pub block_size: Histogram,
    
    // Transaction metrics
    pub transactions_processed: IntCounter,
    pub transactions_in_mempool: IntGauge,
    pub transaction_processing_time: Histogram,
    pub transaction_fees: Histogram,
    
    // Consensus metrics
    pub consensus_rounds: IntCounter,
    pub view_changes: IntCounter,
    pub consensus_latency: Histogram,
    pub validator_votes: IntCounter,
    
    // Network metrics
    pub connected_peers: IntGauge,
    pub messages_sent: IntCounter,
    pub messages_received: IntCounter,
    pub bytes_sent: IntCounter,
    pub bytes_received: IntCounter,
    pub network_latency: Histogram,
    
    // Storage metrics
    pub storage_size: IntGauge,
    pub storage_operations: IntCounter,
    pub storage_latency: Histogram,
    
    // System metrics
    pub cpu_usage: Gauge,
    pub memory_usage: IntGauge,
    pub disk_usage: IntGauge,
    
    registry: Registry,
}

impl NodeMetrics {
    /// Create new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        // Block metrics
        let blocks_processed = IntCounter::new(
            "blockchain_blocks_processed_total",
            "Total number of blocks processed"
        )?;
        registry.register(Box::new(blocks_processed.clone()))?;
        
        let block_height = IntGauge::new(
            "blockchain_block_height",
            "Current block height"
        )?;
        registry.register(Box::new(block_height.clone()))?;
        
        let block_processing_time = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_block_processing_seconds",
                "Time spent processing blocks"
            ).buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0])
        )?;
        registry.register(Box::new(block_processing_time.clone()))?;
        
        let block_size = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_block_size_bytes",
                "Size of processed blocks in bytes"
            ).buckets(vec![1024.0, 10240.0, 102400.0, 1024000.0, 10240000.0])
        )?;
        registry.register(Box::new(block_size.clone()))?;
        
        // Transaction metrics
        let transactions_processed = IntCounter::new(
            "blockchain_transactions_processed_total",
            "Total number of transactions processed"
        )?;
        registry.register(Box::new(transactions_processed.clone()))?;
        
        let transactions_in_mempool = IntGauge::new(
            "blockchain_mempool_transactions",
            "Number of transactions in mempool"
        )?;
        registry.register(Box::new(transactions_in_mempool.clone()))?;
        
        let transaction_processing_time = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_transaction_processing_seconds",
                "Time spent processing transactions"
            ).buckets(vec![0.0001, 0.001, 0.01, 0.1, 1.0])
        )?;
        registry.register(Box::new(transaction_processing_time.clone()))?;
        
        let transaction_fees = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_transaction_fees",
                "Transaction fees"
            ).buckets(vec![1.0, 10.0, 100.0, 1000.0, 10000.0])
        )?;
        registry.register(Box::new(transaction_fees.clone()))?;
        
        // Consensus metrics
        let consensus_rounds = IntCounter::new(
            "blockchain_consensus_rounds_total",
            "Total number of consensus rounds"
        )?;
        registry.register(Box::new(consensus_rounds.clone()))?;
        
        let view_changes = IntCounter::new(
            "blockchain_view_changes_total",
            "Total number of view changes"
        )?;
        registry.register(Box::new(view_changes.clone()))?;
        
        let consensus_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_consensus_latency_seconds",
                "Consensus round latency"
            ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0])
        )?;
        registry.register(Box::new(consensus_latency.clone()))?;
        
        let validator_votes = IntCounter::new(
            "blockchain_validator_votes_total",
            "Total number of validator votes"
        )?;
        registry.register(Box::new(validator_votes.clone()))?;
        
        // Network metrics
        let connected_peers = IntGauge::new(
            "blockchain_connected_peers",
            "Number of connected peers"
        )?;
        registry.register(Box::new(connected_peers.clone()))?;
        
        let messages_sent = IntCounter::new(
            "blockchain_messages_sent_total",
            "Total number of messages sent"
        )?;
        registry.register(Box::new(messages_sent.clone()))?;
        
        let messages_received = IntCounter::new(
            "blockchain_messages_received_total",
            "Total number of messages received"
        )?;
        registry.register(Box::new(messages_received.clone()))?;
        
        let bytes_sent = IntCounter::new(
            "blockchain_bytes_sent_total",
            "Total bytes sent"
        )?;
        registry.register(Box::new(bytes_sent.clone()))?;
        
        let bytes_received = IntCounter::new(
            "blockchain_bytes_received_total",
            "Total bytes received"
        )?;
        registry.register(Box::new(bytes_received.clone()))?;
        
        let network_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_network_latency_seconds",
                "Network message latency"
            ).buckets(vec![0.001, 0.01, 0.1, 1.0, 10.0])
        )?;
        registry.register(Box::new(network_latency.clone()))?;
        
        // Storage metrics
        let storage_size = IntGauge::new(
            "blockchain_storage_size_bytes",
            "Storage size in bytes"
        )?;
        registry.register(Box::new(storage_size.clone()))?;
        
        let storage_operations = IntCounter::new(
            "blockchain_storage_operations_total",
            "Total storage operations"
        )?;
        registry.register(Box::new(storage_operations.clone()))?;
        
        let storage_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "blockchain_storage_latency_seconds",
                "Storage operation latency"
            ).buckets(vec![0.0001, 0.001, 0.01, 0.1, 1.0])
        )?;
        registry.register(Box::new(storage_latency.clone()))?;
        
        // System metrics
        let cpu_usage = Gauge::new(
            "blockchain_cpu_usage_percent",
            "CPU usage percentage"
        )?;
        registry.register(Box::new(cpu_usage.clone()))?;
        
        let memory_usage = IntGauge::new(
            "blockchain_memory_usage_bytes",
            "Memory usage in bytes"
        )?;
        registry.register(Box::new(memory_usage.clone()))?;
        
        let disk_usage = IntGauge::new(
            "blockchain_disk_usage_bytes",
            "Disk usage in bytes"
        )?;
        registry.register(Box::new(disk_usage.clone()))?;
        
        Ok(Self {
            blocks_processed,
            block_height,
            block_processing_time,
            block_size,
            transactions_processed,
            transactions_in_mempool,
            transaction_processing_time,
            transaction_fees,
            consensus_rounds,
            view_changes,
            consensus_latency,
            validator_votes,
            connected_peers,
            messages_sent,
            messages_received,
            bytes_sent,
            bytes_received,
            network_latency,
            storage_size,
            storage_operations,
            storage_latency,
            cpu_usage,
            memory_usage,
            disk_usage,
            registry,
        })
    }
    
    /// Export metrics in Prometheus format
    pub fn export(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
    
    /// Update system metrics
    pub fn update_system_metrics(&self) -> Result<()> {
        // This is simplified - in practice, you'd use system monitoring libraries
        // like sysinfo or procfs to get actual system metrics
        
        // Placeholder values
        self.cpu_usage.set(25.0);
        self.memory_usage.set(1024 * 1024 * 512); // 512MB
        self.disk_usage.set(1024 * 1024 * 1024 * 10); // 10GB
        
        Ok(())
    }
}

/// Metrics server for exposing Prometheus metrics
pub struct MetricsServer {
    port: u16,
    metrics: Arc<NodeMetrics>,
    is_running: Arc<RwLock<bool>>,
}

impl MetricsServer {
    /// Create new metrics server
    pub fn new(port: u16) -> Result<Self> {
        let metrics = Arc::new(NodeMetrics::new()?);
        
        Ok(Self {
            port,
            metrics,
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the metrics server
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut is_running = self.is_running.write().await;
            *is_running = true;
        }
        
        let metrics = self.metrics.clone();
        let is_running = self.is_running.clone();
        let port = self.port;
        
        tokio::spawn(async move {
            let app = warp::path("metrics")
                .map(move || {
                    match metrics.export() {
                        Ok(metrics_text) => {
                            warp::reply::with_header(
                                metrics_text,
                                "content-type",
                                "text/plain; version=0.0.4; charset=utf-8"
                            )
                        }
                        Err(_) => {
                            warp::reply::with_header(
                                "Error exporting metrics".to_string(),
                                "content-type",
                                "text/plain"
                            )
                        }
                    }
                });
            
            // This is simplified - in practice, you'd use a proper HTTP server
            // warp::serve(app).run(([0, 0, 0, 0], port)).await;
        });
        
        // Start periodic system metrics update
        let metrics = self.metrics.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                {
                    let running = is_running.read().await;
                    if !*running {
                        break;
                    }
                }
                
                if let Err(e) = metrics.update_system_metrics() {
                    tracing::warn!("Failed to update system metrics: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Shutdown the metrics server
    pub async fn shutdown(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }
    
    /// Get metrics reference
    pub fn metrics(&self) -> &Arc<NodeMetrics> {
        &self.metrics
    }
}
