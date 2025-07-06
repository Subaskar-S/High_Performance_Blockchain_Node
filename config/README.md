# ⚙️ Configuration

This directory contains configuration files for the blockchain node project.

## 📋 **Configuration Files**

### **🌱 Genesis Configuration**

| File | Description |
|------|-------------|
| [`genesis.json`](genesis.json) | **Genesis blockchain configuration** - Defines initial blockchain state, validators, and network parameters |

## 🌱 **Genesis Configuration (`genesis.json`)**

The genesis file defines the initial state of the blockchain network.

### **Configuration Sections**

#### **🔗 Chain Parameters**
```json
{
  "chain_id": "blockchain-testnet",
  "genesis_time": "2024-01-01T00:00:00Z",
  "initial_height": 0
}
```

- **`chain_id`**: Unique identifier for the blockchain network
- **`genesis_time`**: Timestamp when the blockchain starts
- **`initial_height`**: Starting block height (usually 0)

#### **⚙️ Consensus Parameters**
```json
{
  "consensus_params": {
    "block_time_ms": 1000,
    "max_block_size": 10485760,
    "max_transactions_per_block": 1000,
    "byzantine_fault_tolerance": true
  }
}
```

- **`block_time_ms`**: Target time between blocks in milliseconds
- **`max_block_size`**: Maximum block size in bytes
- **`max_transactions_per_block`**: Maximum transactions per block
- **`byzantine_fault_tolerance`**: Enable BFT consensus

#### **👥 Validator Set**
```json
{
  "validators": [
    {
      "node_id": "validator-1",
      "public_key": "0x1234...",
      "voting_power": 100,
      "address": "validator1@localhost:26656"
    }
  ]
}
```

- **`node_id`**: Unique validator identifier
- **`public_key`**: Validator's public key for signing
- **`voting_power`**: Relative voting weight
- **`address`**: Network address for communication

#### **💰 Initial Accounts**
```json
{
  "initial_accounts": [
    {
      "address": "0x1111111111111111111111111111111111111111",
      "balance": 1000000000000,
      "nonce": 0
    }
  ]
}
```

- **`address`**: Account address
- **`balance`**: Initial account balance
- **`nonce`**: Starting transaction nonce

#### **🏛️ Application State**
```json
{
  "app_state": {
    "native_token": {
      "name": "BlockchainCoin",
      "symbol": "BCC",
      "decimals": 18,
      "total_supply": 21000000000000000000000000
    },
    "governance": {
      "voting_period": 604800,
      "min_deposit": 1000000000000000000,
      "quorum": "0.334"
    }
  }
}
```

- **`native_token`**: Native blockchain token configuration
- **`governance`**: Governance system parameters

## 🔧 **Configuration Usage**

### **Default Configuration**
The blockchain node uses `config/genesis.json` by default:

```bash
# Uses config/genesis.json automatically
cargo run --release -- --node-id validator-1 --mode validator
```

### **Custom Configuration**
Specify a different genesis file:

```bash
# Use custom genesis file
cargo run --release -- --genesis-file /path/to/custom-genesis.json
```

### **Environment-Specific Configurations**

#### **Development Configuration**
```json
{
  "chain_id": "blockchain-dev",
  "consensus_params": {
    "block_time_ms": 500,
    "max_block_size": 1048576
  }
}
```

#### **Testnet Configuration**
```json
{
  "chain_id": "blockchain-testnet",
  "consensus_params": {
    "block_time_ms": 2000,
    "max_block_size": 5242880
  }
}
```

#### **Production Configuration**
```json
{
  "chain_id": "blockchain-mainnet",
  "consensus_params": {
    "block_time_ms": 1000,
    "max_block_size": 10485760
  }
}
```

## 🛠️ **Configuration Management**

### **Creating Custom Genesis Files**

#### **1. Copy Template**
```bash
# Copy the default genesis file
cp config/genesis.json config/my-network-genesis.json
```

#### **2. Modify Parameters**
Edit the copied file to customize:
- Chain ID for your network
- Validator set for your nodes
- Initial account balances
- Consensus parameters

#### **3. Validate Configuration**
```bash
# Test with custom genesis
cargo run --release -- --genesis-file config/my-network-genesis.json --node-id test-node --dev-mode
```

### **Multi-Network Setup**
```
config/
├── genesis.json              # Default configuration
├── dev-genesis.json          # Development network
├── testnet-genesis.json      # Test network
├── staging-genesis.json      # Staging network
└── mainnet-genesis.json      # Production network
```

## 🔒 **Security Considerations**

### **Validator Keys**
- **Never commit real private keys** to version control
- Use placeholder keys in example configurations
- Generate unique keys for each network

### **Network Security**
- Use unique chain IDs for different networks
- Validate all configuration parameters
- Secure validator communication addresses

### **Access Control**
- Restrict access to production genesis files
- Use environment variables for sensitive data
- Implement configuration validation

## 📊 **Configuration Validation**

### **Required Fields**
- ✅ `chain_id` must be unique
- ✅ `validators` must have at least 4 nodes for BFT
- ✅ `consensus_params` must be valid
- ✅ `initial_accounts` must have valid addresses

### **Validation Rules**
- Block time must be > 0
- Max block size must be reasonable
- Validator voting power must be > 0
- Account balances must be non-negative

### **Testing Configuration**
```bash
# Validate genesis file
cargo run --release -- --genesis-file config/genesis.json --validate-only

# Test network startup
cargo run --release -- --genesis-file config/genesis.json --node-id test --dev-mode
```

## 🔄 **Configuration Updates**

### **Runtime Configuration**
Some parameters can be updated at runtime:
- Validator set changes (through governance)
- Consensus parameter updates
- Network upgrades

### **Genesis Updates**
Genesis file changes require:
- Network restart
- All nodes using same genesis
- Coordination among validators

## 📞 **Configuration Support**

### **Common Issues**
- **Invalid JSON**: Check syntax with JSON validator
- **Missing Fields**: Ensure all required fields present
- **Invalid Values**: Check parameter ranges and types
- **Network Mismatch**: Ensure all nodes use same genesis

### **Getting Help**
- **Documentation**: See [`../docs/`](../docs/) for detailed guides
- **Examples**: Check example configurations in this directory
- **Issues**: Report configuration problems on GitHub

---

**Proper configuration is essential for blockchain network operation. This directory provides templates and examples for various deployment scenarios, from development to production networks.**
