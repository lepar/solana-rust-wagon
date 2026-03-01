use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndexedTransaction {
    pub id: Uuid,
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<DateTime<Utc>>,
    pub transaction_type: TransactionType,
    pub program_id: String,
    pub accounts: Vec<String>,
    pub data: String,
    pub status: TransactionStatus,
    pub fee: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum TransactionType {
    TokenTransfer,
    TokenMint,
    TokenBurn,
    TokenAccountCreation,
    NftMint,
    NftTransfer,
    NftBurn,
    NftMetadataUpdate,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "lowercase")]
pub enum TransactionStatus {
    Success,
    Failure,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TokenTransfer {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub mint: String,
    pub from_account: String,
    pub to_account: String,
    pub amount: String,
    pub decimals: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NftMetadata {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub mint: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub seller_fee_basis_points: Option<i32>,
    pub creators: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    pub program_ids: Vec<String>,
    pub account_addresses: Vec<String>,
    pub websocket_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    pub database_url: String,
    pub subscription: SubscriptionConfig,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
}
