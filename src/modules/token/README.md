# Token Module

The Token Module provides comprehensive SPL token management functionality on the Solana blockchain.

## Features

- Create new SPL tokens
- Mint tokens to specific addresses
- Burn tokens from accounts
- Transfer tokens between accounts
- Query token balances for accounts
- Query token information

## API Endpoints

### Create Token
```
POST /api/v1/token
```

**Request Body:**
```json
{
  "name": "My Token",
  "symbol": "MTK",
  "decimals": 9,
  "initial_supply": 1000000000
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mint": "TokenMintPublicKey...",
    "decimals": 9,
    "supply": 0
  }
}
```

### Get Token Information
```
GET /api/v1/token/{mint_address}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mint": "TokenMintPublicKey...",
    "decimals": 9,
    "supply": 1000000000
  }
}
```

### Get Token Balance
```
GET /api/v1/balance/{mint_address}/{owner_address}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mint": "TokenMintPublicKey...",
    "owner": "OwnerPublicKey...",
    "balance": 1000000
  }
}
```

### Mint Tokens
```
POST /api/v1/mint
```

**Request Body:**
```json
{
  "mint": "TokenMintPublicKey...",
  "to": "RecipientPublicKey...",
  "amount": 1000000
}
```

**Response:**
```json
{
  "success": true,
  "signature": "TransactionSignature..."
}
```

### Burn Tokens
```
POST /api/v1/burn
```

**Request Body:**
```json
{
  "mint": "TokenMintPublicKey...",
  "from": "AccountPublicKey...",
  "amount": 1000000
}
```

**Response:**
```json
{
  "success": true,
  "signature": "TransactionSignature..."
}
```

### Transfer Tokens
```
POST /api/v1/transfer
```

**Request Body:**
```json
{
  "mint": "TokenMintPublicKey...",
  "from": "SenderPublicKey...",
  "to": "RecipientPublicKey...",
  "amount": 1000000
}
```

**Response:**
```json
{
  "success": true,
  "signature": "TransactionSignature..."
}
```

## Examples

### Using curl

1. **Create a new token:**
```bash
curl -X POST http://localhost:3000/api/v1/token \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Test Token",
    "symbol": "MTT",
    "decimals": 6,
    "initial_supply": 1000000
  }'
```

2. **Mint tokens to an address:**
```bash
curl -X POST http://localhost:3000/api/v1/mint \
  -H "Content-Type: application/json" \
  -d '{
    "mint": "YOUR_MINT_ADDRESS",
    "to": "RECIPIENT_PUBLIC_KEY",
    "amount": 500000
  }'
```

3. **Transfer tokens:**
```bash
curl -X POST http://localhost:3000/api/v1/transfer \
  -H "Content-Type: application/json" \
  -d '{
    "mint": "YOUR_MINT_ADDRESS",
    "from": "SENDER_PUBLIC_KEY",
    "to": "RECIPIENT_PUBLIC_KEY",
    "amount": 100000
  }'
```

4. **Get token balance:**
```bash
curl http://localhost:3000/api/v1/balance/YOUR_MINT_ADDRESS/OWNER_PUBLIC_KEY
```

5. **Get token information:**
```bash
curl http://localhost:3000/api/v1/token/YOUR_MINT_ADDRESS
```

## Error Handling

All endpoints return consistent error responses:

```json
{
  "success": false,
  "error": "Error description"
}
```

Common error scenarios:
- Invalid public key formats
- Insufficient funds for transaction fees
- Token account doesn't exist
- Invalid token amounts
- Network connectivity issues
