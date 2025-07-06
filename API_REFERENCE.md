# 📡 API Reference

This document provides comprehensive documentation for the blockchain node's JSON-RPC API.

## 🌐 **API Overview**

### **Endpoint Information**
- **Protocol**: JSON-RPC 2.0
- **Transport**: HTTP/HTTPS
- **Default Port**: 8545
- **Content-Type**: `application/json`

### **Request Format**
```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": {...},
  "id": 1
}
```

### **Response Format**
```json
{
  "jsonrpc": "2.0",
  "result": {...},
  "id": 1
}
```

### **Error Format**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Error description",
    "data": {...}
  },
  "id": 1
}
```

## 📦 **Block Methods**

### **blockchain_getBlockByHeight**
Retrieves a block by its height.

**Parameters**:
- `height` (integer): Block height

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getBlockByHeight",
    "params": 1,
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "height": 1,
    "hash": "0x1234567890abcdef...",
    "previous_hash": "0x0000000000000000...",
    "timestamp": 1640995200000,
    "proposer": "validator-1",
    "transaction_count": 5,
    "size": 2048
  },
  "id": 1
}
```

### **blockchain_getBlockByHash**
Retrieves a block by its hash.

**Parameters**:
- `hash` (string): Block hash in hexadecimal format

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getBlockByHash",
    "params": "0x1234567890abcdef...",
    "id": 1
  }'
```

### **blockchain_getLatestBlock**
Retrieves the latest block in the blockchain.

**Parameters**: None

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getLatestBlock",
    "params": {},
    "id": 1
  }'
```

## 💰 **Transaction Methods**

### **blockchain_getTransaction**
Retrieves a transaction by its ID.

**Parameters**:
- `transaction_id` (string): Transaction UUID

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getTransaction",
    "params": "550e8400-e29b-41d4-a716-446655440000",
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "from": "0x1111111111111111111111111111111111111111",
    "to": "0x2222222222222222222222222222222222222222",
    "amount": 1000,
    "fee": 10,
    "nonce": 1,
    "timestamp": 1640995200000,
    "status": "confirmed"
  },
  "id": 1
}
```

### **blockchain_sendTransaction**
Submits a new transaction to the network.

**Parameters**:
- `from` (string): Sender address
- `to` (string): Recipient address
- `amount` (integer): Transfer amount
- `fee` (integer): Transaction fee
- `nonce` (integer): Account nonce
- `signature` (string): Transaction signature
- `data` (string, optional): Additional data

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_sendTransaction",
    "params": {
      "from": "0x1111111111111111111111111111111111111111",
      "to": "0x2222222222222222222222222222222222222222",
      "amount": 1000,
      "fee": 10,
      "nonce": 1,
      "signature": "0x...",
      "data": ""
    },
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "transaction_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "pending"
  },
  "id": 1
}
```

## 👤 **Account Methods**

### **blockchain_getBalance**
Retrieves the balance of an account.

**Parameters**:
- `address` (string): Account address

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getBalance",
    "params": "0x1111111111111111111111111111111111111111",
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "address": "0x1111111111111111111111111111111111111111",
    "balance": 1000000,
    "nonce": 5
  },
  "id": 1
}
```

### **blockchain_getAccountHistory**
Retrieves transaction history for an account.

**Parameters**:
- `address` (string): Account address
- `limit` (integer, optional): Maximum number of transactions (default: 10)
- `offset` (integer, optional): Pagination offset (default: 0)

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getAccountHistory",
    "params": {
      "address": "0x1111111111111111111111111111111111111111",
      "limit": 5,
      "offset": 0
    },
    "id": 1
  }'
```

## 🌐 **Network Methods**

### **blockchain_getNodeStatus**
Retrieves the current status of the blockchain node.

**Parameters**: None

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getNodeStatus",
    "params": {},
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "node_id": "validator-1",
    "is_validator": true,
    "current_height": 1000,
    "current_view": 5,
    "current_round": 1000,
    "connected_peers": 4,
    "mempool_size": 150,
    "is_syncing": false
  },
  "id": 1
}
```

### **blockchain_getPeers**
Retrieves information about connected peers.

**Parameters**: None

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getPeers",
    "params": {},
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "node_id": "validator-2",
      "multiaddr": "/ip4/192.168.1.100/tcp/8000",
      "is_validator": true,
      "last_seen": 1640995200000
    },
    {
      "node_id": "validator-3",
      "multiaddr": "/ip4/192.168.1.101/tcp/8000",
      "is_validator": true,
      "last_seen": 1640995195000
    }
  ],
  "id": 1
}
```

## 🏊 **Mempool Methods**

### **blockchain_getMempoolInfo**
Retrieves information about the transaction mempool.

**Parameters**: None

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getMempoolInfo",
    "params": {},
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "total_transactions": 150,
    "total_added": 1000,
    "total_removed": 850,
    "total_rejected": 50,
    "avg_fee": 15,
    "pending_by_fee": {
      "10": 50,
      "15": 75,
      "20": 25
    }
  },
  "id": 1
}
```

### **blockchain_getMempoolTransactions**
Retrieves pending transactions from the mempool.

**Parameters**:
- `limit` (integer, optional): Maximum number of transactions (default: 10)

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getMempoolTransactions",
    "params": {
      "limit": 5
    },
    "id": 1
  }'
```

## 📊 **Statistics Methods**

### **blockchain_getChainStats**
Retrieves blockchain statistics.

**Parameters**: None

**Example Request**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "blockchain_getChainStats",
    "params": {},
    "id": 1
  }'
```

**Example Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "total_blocks": 1000,
    "total_transactions": 50000,
    "total_accounts": 1000,
    "avg_block_time": 1000,
    "avg_transactions_per_block": 50,
    "chain_size_bytes": 104857600
  },
  "id": 1
}
```

## ⚠️ **Error Codes**

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Invalid JSON-RPC request |
| -32601 | Method not found | Method does not exist |
| -32602 | Invalid params | Invalid method parameters |
| -32603 | Internal error | Internal JSON-RPC error |
| -32000 | Server error | Generic server error |
| -32001 | Not found | Resource not found |
| -32002 | Invalid transaction | Transaction validation failed |
| -32003 | Insufficient balance | Account has insufficient balance |
| -32004 | Invalid nonce | Transaction nonce is invalid |
| -32005 | Network error | Network communication error |

## 🔐 **Authentication**

### **API Key Authentication** (Optional)
If API key authentication is enabled:

```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{...}'
```

### **Rate Limiting**
- **Default Limit**: 100 requests per minute per IP
- **Headers**: 
  - `X-RateLimit-Limit`: Request limit
  - `X-RateLimit-Remaining`: Remaining requests
  - `X-RateLimit-Reset`: Reset timestamp

## 📝 **Best Practices**

### **Request Optimization**
1. **Batch Requests**: Use JSON-RPC batch requests for multiple operations
2. **Caching**: Cache frequently accessed data
3. **Pagination**: Use limit/offset for large result sets
4. **Error Handling**: Always handle error responses

### **Example Batch Request**
```json
[
  {
    "jsonrpc": "2.0",
    "method": "blockchain_getLatestBlock",
    "params": {},
    "id": 1
  },
  {
    "jsonrpc": "2.0",
    "method": "blockchain_getNodeStatus",
    "params": {},
    "id": 2
  }
]
```

### **WebSocket Support** (Future)
Real-time updates via WebSocket subscriptions:

```javascript
const ws = new WebSocket('ws://localhost:8546');

ws.send(JSON.stringify({
  "jsonrpc": "2.0",
  "method": "blockchain_subscribe",
  "params": ["newBlocks"],
  "id": 1
}));
```

This API reference provides comprehensive documentation for interacting with the blockchain node programmatically.
