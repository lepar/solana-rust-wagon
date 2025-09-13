use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint: String,
    pub decimals: u8,
    pub supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintRequest {
    pub mint: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BurnRequest {
    pub mint: String,
    pub from: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    pub mint: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
}
