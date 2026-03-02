# Indexer Module

A real-time blockchain transaction indexer for Solana that monitors and stores token and NFT transactions in a PostgreSQL database.

## 🚀 Features

- **Real-time Indexing**: WebSocket subscriptions to Solana blockchain events
- **Database Storage**: PostgreSQL-based persistent storage with optimized schemas
- **Transaction Tracking**: Support for SPL Token and Metaplex Metadata programs
- **REST API**: Query endpoints for indexed transaction data
- **Background Processing**: Continuous indexing with health monitoring
- **Extensible**: Easy to add new program subscriptions

## 📋 Prerequisites

- PostgreSQL database server
- Database connection string configured in environment variables
- Sufficient disk space for transaction storage

## 🗄️ Database Schema

The indexer creates several tables to store transaction data:

### `indexed_transactions`
Stores all indexed blockchain transactions with metadata.

### `token_transfers`
Detailed token transfer information extracted from transactions.

### `nft_metadata`
NFT metadata including name, symbol, URI, and creator information.

### `subscription_configs`
Configuration for active program and account subscriptions.

## 🔧 Configuration

### Environment Variables

```bash
# Database Configuration (Required)
DATABASE_URL=postgresql://username:password@localhost:5432/solana_indexer

# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
SOLANA_WEBSOCKET_URL=wss://api.mainnet-beta.solana.com
```

### Default Subscriptions

The indexer automatically creates default subscriptions for these programs:
- **SPL Token Program**: `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`
- **Metaplex Metadata**: `metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s`

These are created through the same dynamic subscription system and can be managed via the API.

## � Dynamic Subscription Management

The indexer supports real-time subscription management without requiring server restarts. Each subscription runs in its own isolated async task with dedicated WebSocket connections.

### How It Works

1. **API Request**: You send a POST request to create a subscription
2. **Database Storage**: Configuration is saved to the database
3. **Automatic Spawning**: A new `tokio::spawn()` task is created immediately
4. **WebSocket Connection**: Each subscription connects independently
5. **Event Processing**: Transactions are sent to a shared processing channel

### Key Benefits

- **Zero Downtime**: Add/remove subscriptions without restarting
- **Isolation**: Each subscription runs in its own task
- **Scalability**: Support unlimited concurrent subscriptions
- **Monitoring**: Real-time status of each subscription
- **Persistence**: Subscriptions survive server restarts

### Subscription Lifecycle

```
API Request → Database → Spawn Task → WebSocket Connect → Event Streaming → Transaction Processing
```

## � API Endpoints

### Health Check
```http
GET /api/v1/indexer/health
```

### Subscription Management
```http
POST   /api/v1/indexer/subscriptions     # Create new subscription
GET    /api/v1/indexer/subscriptions     # List all subscriptions
GET    /api/v1/indexer/subscriptions/{id} # Get specific subscription
DELETE /api/v1/indexer/subscriptions/{id} # Remove subscription
```

### Query Transactions
```http
GET /api/v1/indexer/transactions?program_id={program_id}&limit={limit}&offset={offset}
```

**Query Parameters:**
- `program_id` (optional): Filter by specific program ID
- `transaction_type` (optional): Filter by transaction type
- `limit` (optional): Maximum results (default: 50, max: 100)
- `offset` (optional): Pagination offset (default: 0)

### Get Transaction by Signature
```http
GET /api/v1/indexer/transactions/{signature}
```

### Get NFT Metadata
```http
GET /api/v1/indexer/nft/{mint}/metadata
```

### Get Active Subscriptions
```http
GET /api/v1/indexer/subscriptions
```

## 📊 Transaction Types

The indexer categorizes transactions into these types:

- **Token Operations**: `token_transfer`, `token_mint`, `token_burn`, `token_account_creation`
- **NFT Operations**: `nft_mint`, `nft_transfer`, `nft_burn`, `nft_metadata_update`
- **Unknown**: Transactions that don't match known patterns

## 🔄 Background Processing

The indexer runs as a background job with these features:

- **Automatic Reconnection**: Handles WebSocket connection failures
- **Health Monitoring**: Periodic status checks
- **Batch Processing**: Efficient database operations
- **Error Recovery**: Graceful handling of parsing errors

## 📈 Performance Considerations

### Database Indexing
The schema includes optimized indexes for:
- Transaction signatures
- Program IDs
- Account addresses
- Block times
- Transaction types

### Memory Usage
- WebSocket connections use minimal memory
- Database connection pooling prevents connection leaks
- Batch processing reduces memory overhead

## 🛠️ Development

### Running Migrations
```bash
# Migrations run automatically on startup
# To run manually:
sqlx migrate run --database-url "postgresql://username:password@localhost:5432/solana_indexer"
```

### Adding New Program Subscriptions

The indexer supports dynamic subscription management through REST API endpoints. You can add, remove, and monitor subscriptions without restarting the server.

#### Create a New Subscription

```bash
curl -X POST http://localhost:3000/api/v1/indexer/subscriptions \
  -H "Content-Type: application/json" \
  -d '{
    "name": "custom_program",
    "program_ids": ["YourProgramIdHere"],
    "account_addresses": ["AccountAddressHere"],
    "websocket_url": "wss://api.mainnet-beta.solana.com"
  }'
```

**Response:**
```json
{
  "id": "uuid-here",
  "name": "custom_program",
  "program_ids": ["YourProgramIdHere"],
  "account_addresses": ["AccountAddressHere"],
  "websocket_url": "wss://api.mainnet-beta.solana.com",
  "is_running": true,
  "created_at": "2024-01-01T00:00:00Z"
}
```

#### Add Parsing Logic

After creating a subscription, add parsing logic in `indexer_service.rs` for the new program's instructions to properly categorize transactions.

### Monitoring

#### Check Indexer Status
```bash
curl http://localhost:3000/api/v1/indexer/health
```

#### List Active Subscriptions
```bash
curl http://localhost:3000/api/v1/indexer/subscriptions
```

**Response:**
```json
[
  {
    "id": "uuid-here",
    "name": "subscription_uuid-here",
    "program_ids": ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"],
    "account_addresses": [],
    "websocket_url": "wss://api.mainnet-beta.solana.com",
    "is_running": true,
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Get Specific Subscription Status
```bash
curl http://localhost:3000/api/v1/indexer/subscriptions/{subscription_id}
```

#### Remove a Subscription
```bash
curl -X DELETE http://localhost:3000/api/v1/indexer/subscriptions/{subscription_id}
```

#### Monitor Database Growth
```sql
SELECT COUNT(*) FROM indexed_transactions;
SELECT transaction_type, COUNT(*) FROM indexed_transactions GROUP BY transaction_type;
```

## 🔍 Examples

### Create a Custom Program Subscription
```bash
curl -X POST http://localhost:3000/api/v1/indexer/subscriptions \
  -H "Content-Type: application/json" \
  -d '{
    "name": "raydium_program",
    "program_ids": ["675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"],
    "account_addresses": [],
    "websocket_url": "wss://api.mainnet-beta.solana.com"
  }'
```

### Query Recent Token Transfers
```bash
curl "http://localhost:3000/api/v1/indexer/transactions?program_id=TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA&limit=10"
```

### Get NFT Metadata
```bash
curl "http://localhost:3000/api/v1/indexer/nft/{mint_address}/metadata"
```

### Monitor All Active Subscriptions
```bash
curl "http://localhost:3000/api/v1/indexer/subscriptions"
```

### Remove a Subscription
```bash
curl -X DELETE "http://localhost:3000/api/v1/indexer/subscriptions/{subscription_id}"
```

### Monitor Indexer Status
```bash
curl "http://localhost:3000/api/v1/indexer/health"
```

## 🚨 Troubleshooting

### Common Issues

**Database Connection Failed**
- Verify PostgreSQL is running
- Check connection string in `.env` file
- Ensure database exists and user has permissions

**WebSocket Connection Issues**
- Verify network connectivity to Solana RPC
- Check firewall settings
- Consider using a different WebSocket endpoint

**Missing Transactions**
- Verify program subscriptions are active
- Check indexer logs for parsing errors
- Ensure transactions are on the configured network

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

## 🔐 Security Considerations

- Database credentials should be stored securely
- Use read-only database users for API endpoints
- Monitor database size and implement retention policies
- Validate all input parameters in API endpoints

## 📝 Notes

- The indexer stores all transaction data indefinitely
- Consider implementing data archiving for long-running instances
- WebSocket subscriptions may need to be refreshed periodically
- Database performance depends on proper indexing and maintenance
