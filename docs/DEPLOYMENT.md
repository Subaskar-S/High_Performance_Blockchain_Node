# 🚀 Deployment Guide

This guide covers deploying the blockchain node in various environments, from development to production.

## 📋 **Deployment Overview**

### **Deployment Types**
1. **Development**: Single node for testing
2. **Testnet**: Multi-node local network
3. **Staging**: Production-like environment
4. **Production**: Live blockchain network

### **System Requirements**

#### **Minimum Requirements**
- **CPU**: 2 cores, 2.0 GHz
- **RAM**: 4 GB
- **Storage**: 50 GB SSD
- **Network**: 10 Mbps bandwidth
- **OS**: Linux, macOS, or Windows 10+

#### **Recommended Requirements**
- **CPU**: 8 cores, 3.0 GHz
- **RAM**: 16 GB
- **Storage**: 500 GB NVMe SSD
- **Network**: 100 Mbps bandwidth
- **OS**: Ubuntu 20.04+ or CentOS 8+

#### **Production Requirements**
- **CPU**: 16+ cores, 3.5 GHz
- **RAM**: 32+ GB
- **Storage**: 1+ TB NVMe SSD
- **Network**: 1 Gbps bandwidth
- **OS**: Ubuntu 22.04 LTS

## 🔧 **Environment Setup**

### **1. Development Environment**

#### **Prerequisites Installation**
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Python (for scripts)
sudo apt install -y python3 python3-pip
pip3 install requests
```

#### **Project Setup**
```bash
# Clone repository
git clone <repository-url>
cd blockchain-node

# Build project
cargo build --release

# Run tests
cargo test

# Start single node
cargo run --release -- --node-id dev-node --mode validator --dev-mode
```

### **2. Docker Deployment**

#### **Dockerfile**
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/blockchain-node /usr/local/bin/
COPY genesis.json /etc/blockchain/

EXPOSE 8000 8545 9090

ENTRYPOINT ["blockchain-node"]
```

#### **Docker Compose**
```yaml
version: '3.8'

services:
  validator-1:
    build: .
    ports:
      - "8000:8000"
      - "8545:8545"
      - "9090:9090"
    command: >
      --node-id validator-1
      --mode validator
      --listen-addr /ip4/0.0.0.0/tcp/8000
      --rpc-port 8545
      --metrics-port 9090
    volumes:
      - validator1_data:/data
    networks:
      - blockchain

  validator-2:
    build: .
    ports:
      - "8001:8000"
      - "8546:8545"
      - "9091:9090"
    command: >
      --node-id validator-2
      --mode validator
      --listen-addr /ip4/0.0.0.0/tcp/8000
      --rpc-port 8545
      --metrics-port 9090
      --bootstrap-peers /ip4/validator-1/tcp/8000
    volumes:
      - validator2_data:/data
    networks:
      - blockchain
    depends_on:
      - validator-1

volumes:
  validator1_data:
  validator2_data:

networks:
  blockchain:
    driver: bridge
```

### **3. Kubernetes Deployment**

#### **ConfigMap**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: blockchain-config
data:
  genesis.json: |
    {
      "chain_id": "production-mainnet",
      "genesis_time": "2024-01-01T00:00:00Z",
      "validators": [...]
    }
```

#### **StatefulSet**
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: blockchain-validator
spec:
  serviceName: blockchain-validator
  replicas: 4
  selector:
    matchLabels:
      app: blockchain-validator
  template:
    metadata:
      labels:
        app: blockchain-validator
    spec:
      containers:
      - name: blockchain-node
        image: blockchain-node:latest
        ports:
        - containerPort: 8000
          name: p2p
        - containerPort: 8545
          name: rpc
        - containerPort: 9090
          name: metrics
        env:
        - name: NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: RUST_LOG
          value: "info"
        command:
        - blockchain-node
        args:
        - --node-id=$(NODE_ID)
        - --mode=validator
        - --listen-addr=/ip4/0.0.0.0/tcp/8000
        - --rpc-port=8545
        - --metrics-port=9090
        - --db-path=/data
        volumeMounts:
        - name: data
          mountPath: /data
        - name: config
          mountPath: /etc/blockchain
        resources:
          requests:
            cpu: 2
            memory: 4Gi
          limits:
            cpu: 8
            memory: 16Gi
      volumes:
      - name: config
        configMap:
          name: blockchain-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 100Gi
```

## 🌐 **Network Configuration**

### **Firewall Rules**
```bash
# Allow P2P communication
sudo ufw allow 8000/tcp

# Allow RPC access (restrict to trusted IPs in production)
sudo ufw allow from 10.0.0.0/8 to any port 8545

# Allow metrics (restrict to monitoring systems)
sudo ufw allow from 10.0.0.0/8 to any port 9090

# Enable firewall
sudo ufw enable
```

### **Load Balancer Configuration**
```nginx
upstream blockchain_rpc {
    server validator-1:8545;
    server validator-2:8545;
    server validator-3:8545;
    server validator-4:8545;
}

server {
    listen 80;
    server_name blockchain-api.example.com;

    location / {
        proxy_pass http://blockchain_rpc;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## 📊 **Monitoring Setup**

### **Prometheus Configuration**
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'blockchain-nodes'
    static_configs:
      - targets:
        - 'validator-1:9090'
        - 'validator-2:9090'
        - 'validator-3:9090'
        - 'validator-4:9090'
    scrape_interval: 5s
    metrics_path: /metrics
```

### **Grafana Dashboard**
```json
{
  "dashboard": {
    "title": "Blockchain Node Metrics",
    "panels": [
      {
        "title": "Block Height",
        "type": "stat",
        "targets": [
          {
            "expr": "blockchain_block_height",
            "legendFormat": "{{instance}}"
          }
        ]
      },
      {
        "title": "Transactions Per Second",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(blockchain_transactions_processed_total[1m])",
            "legendFormat": "{{instance}}"
          }
        ]
      },
      {
        "title": "Connected Peers",
        "type": "stat",
        "targets": [
          {
            "expr": "blockchain_connected_peers",
            "legendFormat": "{{instance}}"
          }
        ]
      }
    ]
  }
}
```

## 🔒 **Security Configuration**

### **TLS/SSL Setup**
```bash
# Generate certificates
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes

# Configure nginx with SSL
server {
    listen 443 ssl;
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
}
```

### **Access Control**
```bash
# Restrict RPC access to specific IPs
iptables -A INPUT -p tcp --dport 8545 -s 10.0.0.0/8 -j ACCEPT
iptables -A INPUT -p tcp --dport 8545 -j DROP

# Rate limiting
iptables -A INPUT -p tcp --dport 8545 -m limit --limit 25/minute --limit-burst 100 -j ACCEPT
```

## 🔄 **Backup and Recovery**

### **Database Backup**
```bash
#!/bin/bash
# backup_blockchain.sh

BACKUP_DIR="/backup/blockchain"
DATA_DIR="/data/blockchain"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup directory
mkdir -p $BACKUP_DIR

# Stop node gracefully
systemctl stop blockchain-node

# Create backup
tar -czf $BACKUP_DIR/blockchain_backup_$DATE.tar.gz -C $DATA_DIR .

# Restart node
systemctl start blockchain-node

# Clean old backups (keep last 7 days)
find $BACKUP_DIR -name "blockchain_backup_*.tar.gz" -mtime +7 -delete
```

### **Automated Backup with Cron**
```bash
# Add to crontab
0 2 * * * /usr/local/bin/backup_blockchain.sh
```

### **Recovery Process**
```bash
#!/bin/bash
# restore_blockchain.sh

BACKUP_FILE=$1
DATA_DIR="/data/blockchain"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup_file>"
    exit 1
fi

# Stop node
systemctl stop blockchain-node

# Backup current data
mv $DATA_DIR $DATA_DIR.old

# Restore from backup
mkdir -p $DATA_DIR
tar -xzf $BACKUP_FILE -C $DATA_DIR

# Start node
systemctl start blockchain-node
```

## 📈 **Performance Tuning**

### **System Optimization**
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# Optimize network settings
echo "net.core.rmem_max = 134217728" >> /etc/sysctl.conf
echo "net.core.wmem_max = 134217728" >> /etc/sysctl.conf
echo "net.ipv4.tcp_rmem = 4096 87380 134217728" >> /etc/sysctl.conf
echo "net.ipv4.tcp_wmem = 4096 65536 134217728" >> /etc/sysctl.conf

# Apply settings
sysctl -p
```

### **RocksDB Tuning**
```rust
// In storage configuration
let mut opts = Options::default();
opts.set_max_background_jobs(8);
opts.set_max_subcompactions(4);
opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
opts.set_max_write_buffer_number(4);
opts.set_target_file_size_base(256 * 1024 * 1024); // 256MB
```

## 🚨 **Troubleshooting**

### **Common Issues**

#### **Node Won't Start**
```bash
# Check logs
journalctl -u blockchain-node -f

# Check configuration
blockchain-node --help

# Verify permissions
ls -la /data/blockchain
```

#### **Consensus Issues**
```bash
# Check peer connections
curl http://localhost:8545 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"blockchain_getPeers","params":{},"id":1}'

# Check consensus status
curl http://localhost:8545 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"blockchain_getNodeStatus","params":{},"id":1}'
```

#### **Performance Issues**
```bash
# Monitor system resources
htop
iotop
nethogs

# Check metrics
curl http://localhost:9090/metrics
```

### **Log Analysis**
```bash
# Filter consensus logs
journalctl -u blockchain-node | grep "consensus"

# Monitor transaction processing
journalctl -u blockchain-node | grep "transaction"

# Check network issues
journalctl -u blockchain-node | grep "network"
```

This deployment guide provides comprehensive instructions for deploying the blockchain node in various environments with proper monitoring, security, and maintenance procedures.
