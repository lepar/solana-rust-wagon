use anyhow::Result;
use sqlx::{PgPool, Row};
use std::env;
use crate::modules::indexer::models::{
    IndexedTransaction, TokenTransfer, NftMetadata, 
    TransactionType, TransactionStatus, SubscriptionConfig
};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;
        
        Ok(Database { pool })
    }

    pub async fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable not set"))?;
        Self::new(&database_url).await
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    // Transaction operations
    pub async fn store_transaction(&self, transaction: &IndexedTransaction) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO indexed_transactions 
            (id, signature, slot, block_time, transaction_type, program_id, accounts, data, status, fee)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (signature) DO NOTHING
            "#,
            transaction.id,
            transaction.signature,
            transaction.slot,
            transaction.block_time,
            transaction.transaction_type as TransactionType,
            transaction.program_id,
            &transaction.accounts,
            transaction.data,
            transaction.status as TransactionStatus,
            transaction.fee
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_transaction_by_signature(&self, signature: &str) -> Result<Option<IndexedTransaction>> {
        let row = sqlx::query!(
            r#"
            SELECT id, signature, slot, block_time, transaction_type, program_id, accounts, data, status, fee, created_at, updated_at
            FROM indexed_transactions
            WHERE signature = $1
            "#,
            signature
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| IndexedTransaction {
            id: row.id,
            signature: row.signature,
            slot: row.slot,
            block_time: row.block_time,
            transaction_type: row.transaction_type,
            program_id: row.program_id,
            accounts: row.accounts,
            data: row.data,
            status: row.status,
            fee: row.fee,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    // Token transfer operations
    pub async fn store_token_transfer(&self, transfer: &TokenTransfer) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO token_transfers 
            (id, transaction_id, mint, from_account, to_account, amount, decimals)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            transfer.id,
            transfer.transaction_id,
            transfer.mint,
            transfer.from_account,
            transfer.to_account,
            transfer.amount,
            transfer.decimals as i16
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // NFT metadata operations
    pub async fn store_nft_metadata(&self, metadata: &NftMetadata) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO nft_metadata 
            (id, transaction_id, mint, name, symbol, uri, seller_fee_basis_points, creators)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (mint) DO UPDATE SET
                name = EXCLUDED.name,
                symbol = EXCLUDED.symbol,
                uri = EXCLUDED.uri,
                seller_fee_basis_points = EXCLUDED.seller_fee_basis_points,
                creators = EXCLUDED.creators,
                updated_at = NOW()
            "#,
            metadata.id,
            metadata.transaction_id,
            metadata.mint,
            metadata.name,
            metadata.symbol,
            metadata.uri,
            metadata.seller_fee_basis_points,
            metadata.creators
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Subscription config operations
    pub async fn get_subscription_configs(&self) -> Result<Vec<SubscriptionConfig>> {
        let rows = sqlx::query!(
            r#"
            SELECT name, program_ids, account_addresses, websocket_url
            FROM subscription_configs
            WHERE is_active = true
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| SubscriptionConfig {
            program_ids: row.program_ids,
            account_addresses: row.account_addresses,
            websocket_url: row.websocket_url,
        }).collect())
    }

    pub async fn get_transactions_by_program(
        &self, 
        program_id: &str, 
        limit: i64, 
        offset: i64
    ) -> Result<Vec<IndexedTransaction>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, signature, slot, block_time, transaction_type, program_id, accounts, data, status, fee, created_at, updated_at
            FROM indexed_transactions
            WHERE program_id = $1
            ORDER BY block_time DESC
            LIMIT $2 OFFSET $3
            "#,
            program_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| IndexedTransaction {
            id: row.id,
            signature: row.signature,
            slot: row.slot,
            block_time: row.block_time,
            transaction_type: row.transaction_type,
            program_id: row.program_id,
            accounts: row.accounts,
            data: row.data,
            status: row.status,
            fee: row.fee,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }).collect())
    }

    pub async fn get_nft_metadata_by_mint(&self, mint: &str) -> Result<Option<NftMetadata>> {
        let row = sqlx::query!(
            r#"
            SELECT id, transaction_id, mint, name, symbol, uri, seller_fee_basis_points, creators, created_at, updated_at
            FROM nft_metadata
            WHERE mint = $1
            "#,
            mint
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| NftMetadata {
            id: row.id,
            transaction_id: row.transaction_id,
            mint: row.mint,
            name: row.name,
            symbol: row.symbol,
            uri: row.uri,
            seller_fee_basis_points: row.seller_fee_basis_points,
            creators: row.creators,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }
}
