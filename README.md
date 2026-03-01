# Solana Wagon

A modular Rust-based HTTP API for managing SPL tokens and NFTs on the Solana blockchain. More modules to be added.

## 🏗️ Modular Architecture

Solana Wagon is built with a modular architecture that makes it easy to extend with new functionality:

```
src/modules/
├── token/          # SPL Token management
├── nft/            # NFT management
└── indexer/        # Blockchain transaction indexer
```

Each module is self-contained with its own:
- **Manager**: Core business logic
- **Handlers**: HTTP request handlers  
- **Models**: Data structures
- **Routes**: API endpoints

## 🚀 Available Modules

### 🔹 Token Module (v1.0.0)
- Create new SPL tokens
- Mint tokens to specific addresses
- Burn tokens from accounts
- Transfer tokens between accounts
- Query token balances for accounts
- Query token information

📖 **[View Token Module Documentation](src/modules/token/README.md)**

### 🔹 NFT Module (v1.0.0)
- Create new NFTs
- Mint NFTs to specific addresses
- Transfer NFTs between accounts
- Burn NFTs
- Query NFT information
- Query NFT ownership

📖 **[View NFT Module Documentation](src/modules/nft/README.md)**

### 🔹 Indexer Module (v0.1.0)
- Real-time blockchain transaction indexing
- PostgreSQL database storage
- WebSocket subscriptions to Solana events
- Token and NFT transaction tracking
- RESTful API for querying indexed data
- Background job for continuous indexing

📖 **[View Indexer Module Documentation](src/modules/indexer/README.md)**

## ✨ Features

- **Modular Design**: Easy to add new modules (DeFi, Gaming, etc.)
- **RESTful API**: Clean JSON-based HTTP endpoints
- **Solana Integration**: Full SPL token and NFT support
- **Real-time Indexing**: Live blockchain transaction monitoring
- **Database Storage**: PostgreSQL-based data persistence
- **Devnet Ready**: Configured for Solana devnet testing
- **Extensible**: Simple trait-based module system

## Prerequisites

- Rust 1.70+ installed
- Solana CLI tools installed
- A Solana wallet with SOL for transaction fees

## Setup

### 1. Generate a Payer Keypair

First, you need to create a keypair file for the payer account that will fund transactions:

```bash
solana-keygen new --outfile payer-keypair.json
```

### 2. Fund the Payer Account (Devnet)

For development, fund your payer account with devnet SOL:

```bash
# Set to devnet
solana config set --url https://solana-devnet.gateway.tatum.io

# Airdrop SOL to your payer account
solana airdrop 2 $(solana-keygen pubkey payer-keypair.json)
```

### 3. Environment Configuration

Create a `.env` file from the example:

```bash
cp .env.example .env
```

Configure the required environment variables:

```bash
# Solana Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
PAYER_KEYPAIR_PATH=./payer-keypair.json

# Database Configuration (required for indexer)
DATABASE_URL=postgresql://username:password@localhost:5432/solana_indexer

# Server Configuration
PORT=3000

# Optional: Custom WebSocket URL
SOLANA_WEBSOCKET_URL=wss://api.mainnet-beta.solana.com
```

### 4. Database Setup (Required for Indexer)

The indexer module requires a PostgreSQL database:

```bash
# Install PostgreSQL (Ubuntu/Debian)
sudo apt update
sudo apt install postgresql postgresql-contrib

# Create database
sudo -u postgres createdb solana_indexer

# Create user (optional)
sudo -u postgres createuser --interactive
```

## Running the Application

```bash
cargo run
```

The server will start on `http://localhost:3000` by default.

## 📡 API Endpoints

### Health Check
```
GET /api/v1/health
```

**Response:**
```json
{
  "status": "healthy",
  "service": "solana-token-manager",
  "version": "3.0.0",
  "features": ["create", "mint", "burn", "transfer", "info"]
}
```

### Module Endpoints

For detailed API documentation and examples, see the individual module READMEs:

- **Token Module**: [src/modules/token/README.md](src/modules/token/README.md)
- **NFT Module**: [src/modules/nft/README.md](src/modules/nft/README.md)
- **Indexer Module**: [src/modules/indexer/README.md](src/modules/indexer/README.md)

---

## Quick Start

For detailed examples and usage instructions, see the individual module documentation:

- **Token Module Examples**: [src/modules/token/README.md#examples](src/modules/token/README.md#examples)
- **NFT Module Examples**: [src/modules/nft/README.md#examples](src/modules/nft/README.md#examples)
- **Indexer Module Examples**: [src/modules/indexer/README.md#examples](src/modules/indexer/README.md#examples)

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
- Database connection errors (indexer module)
- WebSocket connection failures (indexer module)
- Network connectivity issues

## 🔧 Extending the Architecture

### Adding New Modules

The modular architecture makes it easy to add new functionality. Here's how to create a new module:

1. **Create module directory:**
```bash
mkdir -p src/modules/your_module
```

2. **Implement the Module trait:**
```rust
// src/modules/your_module/mod.rs
use crate::modules::Module;
use actix_web::web;

pub struct YourModule;

impl Module for YourModule {
    fn configure_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/your-module")
                .route("", web::post().to(handlers::your_handler))
        );
    }

    fn name() -> &'static str {
        "your_module"
    }

    fn version() -> &'static str {
        "1.0.0"
    }
}

pub mod handlers;
pub mod manager;
pub mod models;
```

3. **Add to main modules:**
```rust
// src/modules/mod.rs
pub mod your_module;
```

4. **Configure in server:**
```rust
// src/server.rs
.configure(crate::modules::your_module::YourModule::configure_routes)
```

### Module Structure

Each module should follow this structure:
- `mod.rs` - Module configuration and trait implementation
- `manager.rs` - Core business logic
- `handlers.rs` - HTTP request handlers
- `models.rs` - Data structures and request/response types

## Development

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Running in Release Mode
```bash
cargo run --release
```

## Network Configuration

The application defaults to Solana Devnet for development. To use Mainnet or Testnet:

1. Set the `SOLANA_RPC_URL` environment variable
2. Ensure your payer account has sufficient SOL for transaction fees
3. Update your Solana CLI configuration:

```bash
# For mainnet
solana config set --url https://api.mainnet-beta.solana.com

# For testnet
solana config set --url https://solana-devnet.gateway.tatum.io
```

## Security Considerations

- Keep your payer keypair secure and never commit it to version control
- Use environment variables for sensitive configuration
- Consider using a hardware wallet or secure key management for production
- Monitor your payer account balance to ensure sufficient funds for transactions

## Security Warning

⚠️ **IMPORTANT**: This software handles cryptocurrency transactions. Users are responsible for:
- Securing their private keys
- Following security best practices
- Understanding the risks of blockchain transactions
- Complying with local laws and regulations

The developers are not responsible for lost funds, stolen keys, or user negligence, or any other type of losses that may arise from the use of this code.
This code should be used as references and not as production-based

## License

This project is open source and available under the MIT License.
