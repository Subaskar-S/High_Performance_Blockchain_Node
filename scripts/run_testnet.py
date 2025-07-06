#!/usr/bin/env python3
"""
Script to run a 5-node blockchain testnet for testing and development.
This script starts multiple blockchain nodes and connects them together.
"""

import os
import sys
import time
import json
import subprocess
import signal
import threading
from pathlib import Path

class BlockchainTestnet:
    def __init__(self, num_nodes=5):
        self.num_nodes = num_nodes
        self.nodes = []
        self.base_port = 8000
        self.base_rpc_port = 9000
        self.base_metrics_port = 9100
        self.data_dir = Path("./testnet_data")
        self.running = False
        
    def setup_directories(self):
        """Create data directories for each node."""
        print("Setting up node directories...")
        
        # Clean up existing data
        if self.data_dir.exists():
            import shutil
            shutil.rmtree(self.data_dir)
        
        self.data_dir.mkdir(exist_ok=True)
        
        for i in range(self.num_nodes):
            node_dir = self.data_dir / f"node-{i+1}"
            node_dir.mkdir(exist_ok=True)
            
            # Copy genesis file to each node directory
            genesis_src = Path("config/genesis.json")
            if genesis_src.exists():
                import shutil
                shutil.copy(genesis_src, node_dir / "genesis.json")
    
    def generate_node_config(self, node_id, is_validator=True):
        """Generate configuration for a node."""
        node_num = int(node_id.split('-')[1])
        
        # Calculate ports
        p2p_port = self.base_port + node_num - 1
        rpc_port = self.base_rpc_port + node_num - 1
        metrics_port = self.base_metrics_port + node_num - 1
        
        # Generate bootstrap peers (connect to other nodes)
        bootstrap_peers = []
        for i in range(1, self.num_nodes + 1):
            if i != node_num:
                peer_port = self.base_port + i - 1
                bootstrap_peers.append(f"/ip4/127.0.0.1/tcp/{peer_port}")
        
        config = {
            "node_id": node_id,
            "mode": "validator" if is_validator else "observer",
            "listen_addr": f"/ip4/0.0.0.0/tcp/{p2p_port}",
            "bootstrap_peers": ",".join(bootstrap_peers),
            "db_path": str(self.data_dir / node_id),
            "rpc_port": rpc_port,
            "metrics_port": metrics_port,
            "enable_metrics": True,
            "max_peers": 100,
            "block_time_ms": 2000,  # 2 second blocks for testing
            "mempool_size": 1000,
            "dev_mode": True
        }
        
        return config
    
    def start_node(self, node_id, config):
        """Start a single blockchain node."""
        print(f"Starting {node_id}...")
        
        # Build command line arguments
        cmd = [
            "cargo", "run", "--release", "--",
            "--node-id", config["node_id"],
            "--mode", config["mode"],
            "--listen-addr", config["listen_addr"],
            "--db-path", str(config["db_path"]).replace("\\", "/"),  # Fix Windows paths
            "--rpc-port", str(config["rpc_port"]),
            "--metrics-port", str(config["metrics_port"]),
            "--max-peers", str(config["max_peers"]),
            "--block-time-ms", str(config["block_time_ms"]),
            "--mempool-size", str(config["mempool_size"]),
            "--genesis-file", str(self.data_dir / node_id / "genesis.json").replace("\\", "/")
        ]
        
        if config["bootstrap_peers"]:
            cmd.extend(["--bootstrap-peers", config["bootstrap_peers"]])
        
        if config["enable_metrics"]:
            cmd.append("--enable-metrics")
        
        if config["dev_mode"]:
            cmd.append("--dev-mode")
        
        # Set environment variables
        env = os.environ.copy()
        env["RUST_LOG"] = "info"
        
        # Start the process
        try:
            process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                env=env,
                cwd=Path.cwd()
            )
            
            node_info = {
                "id": node_id,
                "process": process,
                "config": config,
                "started_at": time.time()
            }
            
            self.nodes.append(node_info)
            
            # Start log monitoring thread
            log_thread = threading.Thread(
                target=self.monitor_node_logs,
                args=(node_id, process),
                daemon=True
            )
            log_thread.start()
            
            return True
            
        except Exception as e:
            print(f"Failed to start {node_id}: {e}")
            return False
    
    def monitor_node_logs(self, node_id, process):
        """Monitor and display node logs."""
        while process.poll() is None:
            try:
                line = process.stdout.readline()
                if line:
                    print(f"[{node_id}] {line.decode().strip()}")
            except:
                break
    
    def start_testnet(self):
        """Start the entire testnet."""
        print(f"Starting {self.num_nodes}-node blockchain testnet...")
        
        # Setup directories
        self.setup_directories()
        
        # Start validator nodes
        for i in range(1, self.num_nodes + 1):
            node_id = f"validator-{i}"
            config = self.generate_node_config(node_id, is_validator=True)
            
            if not self.start_node(node_id, config):
                print(f"Failed to start {node_id}")
                return False
            
            # Wait a bit between node starts
            time.sleep(2)
        
        self.running = True
        print(f"Testnet started with {len(self.nodes)} nodes")
        
        # Print connection information
        self.print_node_info()
        
        return True
    
    def print_node_info(self):
        """Print information about running nodes."""
        print("\n" + "="*60)
        print("BLOCKCHAIN TESTNET INFORMATION")
        print("="*60)
        
        for node in self.nodes:
            config = node["config"]
            print(f"\nNode: {config['node_id']}")
            print(f"  Mode: {config['mode']}")
            print(f"  P2P Port: {config['listen_addr'].split('/')[-1]}")
            print(f"  RPC Port: {config['rpc_port']}")
            print(f"  Metrics Port: {config['metrics_port']}")
            print(f"  Data Directory: {config['db_path']}")
            print(f"  RPC Endpoint: http://localhost:{config['rpc_port']}")
            print(f"  Metrics Endpoint: http://localhost:{config['metrics_port']}/metrics")
        
        print(f"\nTestnet Status: {len(self.nodes)} nodes running")
        print("="*60)
    
    def stop_testnet(self):
        """Stop all nodes in the testnet."""
        print("Stopping testnet...")
        
        for node in self.nodes:
            try:
                node["process"].terminate()
                node["process"].wait(timeout=10)
                print(f"Stopped {node['id']}")
            except subprocess.TimeoutExpired:
                print(f"Force killing {node['id']}")
                node["process"].kill()
            except Exception as e:
                print(f"Error stopping {node['id']}: {e}")
        
        self.nodes.clear()
        self.running = False
        print("Testnet stopped")
    
    def wait_for_shutdown(self):
        """Wait for shutdown signal."""
        try:
            while self.running:
                time.sleep(1)
        except KeyboardInterrupt:
            print("\nReceived shutdown signal...")
            self.stop_testnet()
    
    def send_test_transactions(self):
        """Send some test transactions to the network."""
        print("Sending test transactions...")
        
        # This would use the RPC API to send transactions
        # For now, it's a placeholder
        
        import requests
        
        for i, node in enumerate(self.nodes[:3]):  # Send to first 3 nodes
            rpc_port = node["config"]["rpc_port"]
            
            try:
                # Example transaction (this would need proper implementation)
                tx_data = {
                    "jsonrpc": "2.0",
                    "method": "blockchain_sendTransaction",
                    "params": {
                        "from": f"0x{'1' * 40}",
                        "to": f"0x{'2' * 40}",
                        "amount": 1000 + i * 100,
                        "fee": 10
                    },
                    "id": i + 1
                }
                
                response = requests.post(
                    f"http://localhost:{rpc_port}",
                    json=tx_data,
                    timeout=5
                )
                
                if response.status_code == 200:
                    print(f"Sent transaction to {node['id']}")
                else:
                    print(f"Failed to send transaction to {node['id']}: {response.status_code}")
                    
            except Exception as e:
                print(f"Error sending transaction to {node['id']}: {e}")
    
    def check_node_status(self):
        """Check the status of all nodes."""
        print("Checking node status...")
        
        import requests
        
        for node in self.nodes:
            rpc_port = node["config"]["rpc_port"]
            
            try:
                status_request = {
                    "jsonrpc": "2.0",
                    "method": "blockchain_getNodeStatus",
                    "params": {},
                    "id": 1
                }
                
                response = requests.post(
                    f"http://localhost:{rpc_port}",
                    json=status_request,
                    timeout=5
                )
                
                if response.status_code == 200:
                    result = response.json()
                    if "result" in result:
                        status = result["result"]
                        print(f"{node['id']}: Height={status.get('current_height', 'N/A')}, "
                              f"Peers={status.get('connected_peers', 'N/A')}, "
                              f"Mempool={status.get('mempool_size', 'N/A')}")
                    else:
                        print(f"{node['id']}: Error - {result.get('error', 'Unknown')}")
                else:
                    print(f"{node['id']}: HTTP {response.status_code}")
                    
            except Exception as e:
                print(f"{node['id']}: Connection failed - {e}")

def main():
    """Main function."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Run blockchain testnet")
    parser.add_argument("--nodes", type=int, default=5, help="Number of nodes to run")
    parser.add_argument("--test-tx", action="store_true", help="Send test transactions")
    parser.add_argument("--status", action="store_true", help="Check node status and exit")
    
    args = parser.parse_args()
    
    testnet = BlockchainTestnet(num_nodes=args.nodes)
    
    if args.status:
        # Just check status of running nodes
        testnet.check_node_status()
        return
    
    # Setup signal handlers
    def signal_handler(signum, frame):
        print(f"\nReceived signal {signum}")
        testnet.stop_testnet()
        sys.exit(0)
    
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Start the testnet
    if testnet.start_testnet():
        # Wait a bit for nodes to connect
        time.sleep(10)
        
        if args.test_tx:
            testnet.send_test_transactions()
            time.sleep(5)
        
        # Check initial status
        testnet.check_node_status()
        
        print("\nTestnet is running. Press Ctrl+C to stop.")
        testnet.wait_for_shutdown()
    else:
        print("Failed to start testnet")
        sys.exit(1)

if __name__ == "__main__":
    main()
