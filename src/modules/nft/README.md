# NFT Module

The NFT Module provides comprehensive NFT (Non-Fungible Token) management functionality on the Solana blockchain.

## Features

- Create new NFTs
- Mint NFTs to specific addresses
- Transfer NFTs between accounts
- Burn NFTs
- Query NFT information
- Query NFT ownership

## API Endpoints

### Create NFT
```
POST /api/v1/nft
```

**Request Body:**
```json
{
  "name": "My NFT",
  "symbol": "MNFT",
  "uri": "https://example.com/metadata.json",
  "seller_fee_basis_points": 500,
  "creators": [
    {
      "address": "CreatorPublicKey...",
      "verified": true,
      "share": 100
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mint": "NFTPublicKey...",
    "name": "My NFT",
    "symbol": "MNFT",
    "uri": "https://example.com/metadata.json",
    "seller_fee_basis_points": 500,
    "creators": [...]
  }
}
```

### Get NFT Information
```
GET /api/v1/nft/{mint_address}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mint": "NFTPublicKey...",
    "name": "My NFT",
    "symbol": "MNFT",
    "uri": "https://example.com/metadata.json",
    "seller_fee_basis_points": 500,
    "creators": [...]
  }
}
```

### Get NFT Owner
```
GET /api/v1/nft/{mint_address}/owner
```

**Response:**
```json
{
  "success": true,
  "data": {
    "mint": "NFTPublicKey...",
    "owner": "OwnerPublicKey..."
  }
}
```

### Mint NFT
```
POST /api/v1/nft-mint
```

**Request Body:**
```json
{
  "mint": "NFTPublicKey...",
  "to": "RecipientPublicKey..."
}
```

**Response:**
```json
{
  "success": true,
  "signature": "TransactionSignature..."
}
```

### Transfer NFT
```
POST /api/v1/nft-transfer
```

**Request Body:**
```json
{
  "mint": "NFTPublicKey...",
  "from": "SenderPublicKey...",
  "to": "RecipientPublicKey..."
}
```

**Response:**
```json
{
  "success": true,
  "signature": "TransactionSignature..."
}
```

### Burn NFT
```
POST /api/v1/nft-burn
```

**Request Body:**
```json
{
  "mint": "NFTPublicKey...",
  "from": "OwnerPublicKey..."
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

1. **Create a new NFT:**
```bash
curl -X POST http://localhost:3000/api/v1/nft \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My First NFT",
    "symbol": "MFN",
    "uri": "https://example.com/metadata.json",
    "seller_fee_basis_points": 500,
    "creators": [
      {
        "address": "YOUR_PUBLIC_KEY",
        "verified": true,
        "share": 100
      }
    ]
  }'
```

2. **Mint NFT to an address:**
```bash
curl -X POST http://localhost:3000/api/v1/nft-mint \
  -H "Content-Type: application/json" \
  -d '{
    "mint": "YOUR_NFT_MINT_ADDRESS",
    "to": "RECIPIENT_PUBLIC_KEY"
  }'
```

3. **Transfer NFT:**
```bash
curl -X POST http://localhost:3000/api/v1/nft-transfer \
  -H "Content-Type: application/json" \
  -d '{
    "mint": "YOUR_NFT_MINT_ADDRESS",
    "from": "SENDER_PUBLIC_KEY",
    "to": "RECIPIENT_PUBLIC_KEY"
  }'
```

4. **Get NFT information:**
```bash
curl http://localhost:3000/api/v1/nft/YOUR_NFT_MINT_ADDRESS
```

5. **Get NFT owner:**
```bash
curl http://localhost:3000/api/v1/nft/YOUR_NFT_MINT_ADDRESS/owner
```

6. **Burn NFT:**
```bash
curl -X POST http://localhost:3000/api/v1/nft-burn \
  -H "Content-Type: application/json" \
  -d '{
    "mint": "YOUR_NFT_MINT_ADDRESS",
    "from": "OWNER_PUBLIC_KEY"
  }'
```

## NFT Metadata

NFTs use metadata URIs to store additional information. The metadata should be a JSON file containing:

```json
{
  "name": "My NFT",
  "symbol": "MNFT",
  "description": "A description of the NFT",
  "image": "https://example.com/image.png",
  "attributes": [
    {
      "trait_type": "Color",
      "value": "Blue"
    },
    {
      "trait_type": "Rarity",
      "value": "Legendary"
    }
  ]
}
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
- NFT account doesn't exist
- Invalid NFT operations
- Network connectivity issues
