# High-Throughput Blockchain Node

A high-performance, fault-tolerant blockchain node implementation in Rust, designed to handle 1000+ peers and 10,000+ TPS with Byzantine Fault Tolerant (BFT) consensus.

> **📁 Project Reorganized**: Documentation moved to [`docs/`](docs/), configuration to [`config/`](config/), and setup tools to [`tools/`](tools/) for better organization.

## 🚀 Features

- **High Performance**: Designed for 1000+ concurrent connections and 10,000+ TPS
- **Byzantine Fault Tolerance**: PBFT consensus algorithm with view changes
- **Modular Architecture**: Clean separation of concerns with pluggable components
- **P2P Networking**: libp2p-based networking with gossip protocol and peer discovery
- **Persistent Storage**: RocksDB for high-performance data persistence
- **Metrics & Monitoring**: Prometheus metrics for comprehensive observability
- **JSON-RPC API**: Standard API for blockchain interactions
- **Memory Pool**: Priority-based transaction pool with efficient ordering

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   JSON-RPC API  │    │   Metrics       │    │   CLI Interface │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
┌─────────────────────────────────────────────────────────────────┐
│                    Blockchain Node                             │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   Consensus     │    Mempool      │       Validation            │
│   (PBFT)        │   (Priority)    │       (Rules)               │
├─────────────────┼─────────────────┼─────────────────────────────┤
│   Network       │    Storage      │       Types                 │
│   (libp2p)      │   (RocksDB)     │       (Core)                │
└─────────────────┴─────────────────┴─────────────────────────────┘
```

## 📁 Project Structure

```
blockchain-node/
├── src/                 # Source code
│   ├── main.rs              # Application entry point
│   ├── cli.rs               # Command-line interface
│   ├── node.rs              # Main blockchain node orchestrator
│   ├── types.rs             # Core data structures
│   ├── consensus/           # BFT consensus implementation
│   │   ├── mod.rs           # Consensus engine
│   │   ├── pbft.rs          # PBFT algorithm
│   │   ├── leader_election.rs # Leader selection
│   │   └── view_change.rs   # View change protocol
│   ├── network/             # P2P networking layer
│   │   ├── mod.rs           # Network manager
│   │   ├── gossip.rs        # Gossip protocol
│   │   ├── discovery.rs     # Peer discovery
│   │   └── transport.rs     # Transport utilities
│   ├── storage/             # Persistent storage
│   │   ├── mod.rs           # Storage interface
│   │   ├── block_store.rs   # Block storage
│   │   ├── state_store.rs   # State management
│   │   └── transaction_store.rs # Transaction storage
│   ├── mempool.rs           # Transaction pool
│   ├── validation.rs        # Block/transaction validation
│   ├── metrics.rs           # Prometheus metrics
│   └── api.rs               # JSON-RPC API
├── docs/                # Documentation
│   ├── DOCUMENTATION.md     # Complete file documentation
│   ├── ARCHITECTURE.md      # System architecture guide
│   ├── DEPLOYMENT.md        # Deployment instructions
│   ├── API_REFERENCE.md     # JSON-RPC API documentation
│   └── PROJECT_SUMMARY.md   # Project overview
├── config/              # Configuration files
│   └── genesis.json         # Genesis blockchain configuration
├── tools/               # Setup and utility tools
│   ├── setup_windows.bat    # Windows setup script
│   └── setup_windows.ps1    # PowerShell setup script
├── scripts/             # Automation scripts
│   ├── run_testnet.py       # 5-node testnet simulation
│   ├── build_and_test.sh    # Build automation (Linux/Mac)
│   ├── build_and_test.bat   # Build automation (Windows)
│   ├── setup_git.sh         # Git setup (Linux/Mac)
│   └── setup_git.bat        # Git setup (Windows)
├── benches/             # Performance benchmarks
│   ├── consensus_benchmark.rs # Consensus performance tests
│   └── network_benchmark.rs   # Network performance tests
├── .github/             # GitHub configuration
│   └── workflows/
│       └── ci.yml           # CI/CD pipeline
├── README.md            # Main project documentation
├── CONTRIBUTING.md      # Contribution guidelines
├── LICENSE              # MIT license
├── Cargo.toml           # Rust project configuration
└── .gitignore           # Git ignore rules
```

## 🛠️ Installation

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd blockchain-node

# Build the project
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

## 🚀 Quick Start

### Single Node

```bash
# Start a single validator node
cargo run --release -- \
  --node-id validator-1 \
  --mode validator \
  --listen-addr "/ip4/0.0.0.0/tcp/8000" \
  --rpc-port 9000 \
  --metrics-port 9100
```

### Multi-Node Testnet

Use the provided Python script to run a 5-node testnet:

```bash
# Make the script executable
chmod +x scripts/run_testnet.py

# Start 5-node testnet
python3 scripts/run_testnet.py --nodes 5

# Send test transactions
python3 scripts/run_testnet.py --nodes 5 --test-tx

# Check node status
python3 scripts/run_testnet.py --status
```

## 📊 Configuration

### Command Line Options

```bash
blockchain-node [OPTIONS]

OPTIONS:
    -m, --mode <MODE>                    Node operation mode [default: validator]
        --node-id <NODE_ID>              Node identifier [default: node-1]
        --listen-addr <LISTEN_ADDR>      Listen address for P2P [default: /ip4/0.0.0.0/tcp/0]
        --bootstrap-peers <PEERS>        Bootstrap peers (comma-separated)
        --genesis-file <FILE>            Path to genesis file [default: config/genesis.json]
        --db-path <PATH>                 Database path [default: ./data]
        --rpc-port <PORT>                JSON-RPC server port [default: 8545]
        --metrics-port <PORT>            Metrics server port [default: 9090]
        --max-peers <COUNT>              Maximum number of peers [default: 1000]
        --block-time-ms <MS>             Block time in milliseconds [default: 1000]
        --mempool-size <SIZE>            Transaction pool size limit [default: 10000]
        --enable-metrics                 Enable metrics collection [default: true]
        --dev-mode                       Enable development mode [default: false]
```

### Genesis Configuration

The `config/genesis.json` file defines the initial blockchain state:

```json
{
  "chain_id": "blockchain-testnet",
  "genesis_time": "2024-01-01T00:00:00Z",
  "validators": [
    {
      "node_id": "validator-1",
      "public_key": "0x...",
      "voting_power": 100
    }
  ],
  "initial_accounts": [
    {
      "address": "0x1111111111111111111111111111111111111111",
      "balance": 1000000000000
    }
  ]
}
```

## 🔧 API Reference

### JSON-RPC Endpoints

The node exposes a JSON-RPC API on the configured port (default: 8545):

```bash
# Get latest block
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"blockchain_getLatestBlock","params":{},"id":1}'

# Get block by height
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"blockchain_getBlockByHeight","params":1,"id":1}'

# Get node status
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"blockchain_getNodeStatus","params":{},"id":1}'
```

### Metrics

Prometheus metrics are available at `http://localhost:9090/metrics`:

- `blockchain_blocks_processed_total` - Total blocks processed
- `blockchain_transactions_processed_total` - Total transactions processed
- `blockchain_connected_peers` - Number of connected peers
- `blockchain_consensus_latency_seconds` - Consensus round latency
- `blockchain_mempool_transactions` - Transactions in mempool

## 🧪 Testing

### Unit Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test consensus
cargo test storage
cargo test mempool
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run with logging
RUST_LOG=debug cargo test
```

### Performance Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmarks
cargo bench consensus
cargo bench network
```

## 📈 Performance Targets

- **Throughput**: 10,000+ TPS
- **Latency**: Sub-10ms block propagation in LAN
- **Scalability**: 1000+ concurrent peer connections
- **Consensus**: Sub-second finality with BFT
- **Storage**: Efficient block and state storage with RocksDB

## 🔒 Security

- **Byzantine Fault Tolerance**: Tolerates up to f = (n-1)/3 malicious nodes
- **Cryptographic Signatures**: Ed25519 for message authentication
- **Network Security**: Noise protocol for encrypted P2P communication
- **Input Validation**: Comprehensive validation of all inputs

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation for API changes
- Run `cargo fmt` and `cargo clippy` before committing
- Ensure all tests pass

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [libp2p](https://libp2p.io/) for P2P networking
- [RocksDB](https://rocksdb.org/) for storage engine
- [Tokio](https://tokio.rs/) for async runtime
- [Prometheus](https://prometheus.io/) for metrics

## 📞 Support

- Create an issue for bug reports or feature requests
- Join our community discussions
- Check the documentation for common questions

---

**Note**: This is a demonstration blockchain implementation. For production use, additional security audits, testing, and hardening would be required.
