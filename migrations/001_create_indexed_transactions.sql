-- Create indexed_transactions table
CREATE TABLE IF NOT EXISTS indexed_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signature VARCHAR(88) UNIQUE NOT NULL,
    slot BIGINT NOT NULL,
    block_time TIMESTAMP WITH TIME ZONE,
    transaction_type VARCHAR(50) NOT NULL CHECK (transaction_type IN (
        'token_transfer', 'token_mint', 'token_burn', 'token_account_creation',
        'nft_mint', 'nft_transfer', 'nft_burn', 'nft_metadata_update', 'unknown'
    )),
    program_id VARCHAR(44) NOT NULL,
    accounts TEXT[] NOT NULL,
    data TEXT NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'failure', 'pending')),
    fee BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_indexed_transactions_signature ON indexed_transactions(signature);
CREATE INDEX IF NOT EXISTS idx_indexed_transactions_slot ON indexed_transactions(slot);
CREATE INDEX IF NOT EXISTS idx_indexed_transactions_program_id ON indexed_transactions(program_id);
CREATE INDEX IF NOT EXISTS idx_indexed_transactions_transaction_type ON indexed_transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_indexed_transactions_block_time ON indexed_transactions(block_time);
CREATE INDEX IF NOT EXISTS idx_indexed_transactions_accounts ON indexed_transactions USING GIN(accounts);

-- Create token_transfers table
CREATE TABLE IF NOT EXISTS token_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES indexed_transactions(id) ON DELETE CASCADE,
    mint VARCHAR(44) NOT NULL,
    from_account VARCHAR(44) NOT NULL,
    to_account VARCHAR(44) NOT NULL,
    amount VARCHAR(79) NOT NULL, -- Support for large numbers
    decimals SMALLINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for token_transfers
CREATE INDEX IF NOT EXISTS idx_token_transfers_transaction_id ON token_transfers(transaction_id);
CREATE INDEX IF NOT EXISTS idx_token_transfers_mint ON token_transfers(mint);
CREATE INDEX IF NOT EXISTS idx_token_transfers_from_account ON token_transfers(from_account);
CREATE INDEX IF NOT EXISTS idx_token_transfers_to_account ON token_transfers(to_account);

-- Create nft_metadata table
CREATE TABLE IF NOT EXISTS nft_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    transaction_id UUID NOT NULL REFERENCES indexed_transactions(id) ON DELETE CASCADE,
    mint VARCHAR(44) UNIQUE NOT NULL,
    name VARCHAR(255),
    symbol VARCHAR(20),
    uri TEXT,
    seller_fee_basis_points INTEGER,
    creators JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for nft_metadata
CREATE INDEX IF NOT EXISTS idx_nft_metadata_transaction_id ON nft_metadata(transaction_id);
CREATE INDEX IF NOT EXISTS idx_nft_metadata_mint ON nft_metadata(mint);
CREATE INDEX IF NOT EXISTS idx_nft_metadata_name ON nft_metadata(name);
CREATE INDEX IF NOT EXISTS idx_nft_metadata_symbol ON nft_metadata(symbol);

-- Create subscription_configs table for managing active subscriptions
CREATE TABLE IF NOT EXISTS subscription_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    program_ids TEXT[] NOT NULL DEFAULT '{}',
    account_addresses TEXT[] NOT NULL DEFAULT '{}',
    websocket_url VARCHAR(255) NOT NULL DEFAULT 'wss://api.mainnet-beta.solana.com',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for subscription_configs
CREATE INDEX IF NOT EXISTS idx_subscription_configs_name ON subscription_configs(name);
CREATE INDEX IF NOT EXISTS idx_subscription_configs_is_active ON subscription_configs(is_active);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_indexed_transactions_updated_at 
    BEFORE UPDATE ON indexed_transactions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_nft_metadata_updated_at 
    BEFORE UPDATE ON nft_metadata 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_subscription_configs_updated_at 
    BEFORE UPDATE ON subscription_configs 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
