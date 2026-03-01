use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::modules::indexer::database::Database;

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub signature: String,
    pub slot: i64,
    pub block_time: Option<chrono::DateTime<chrono::Utc>>,
    pub transaction_type: String,
    pub program_id: String,
    pub accounts: Vec<String>,
    pub status: String,
    pub fee: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct NftMetadataResponse {
    pub id: Uuid,
    pub mint: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub seller_fee_basis_points: Option<i32>,
    pub creators: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionQuery {
    pub program_id: Option<String>,
    pub transaction_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/indexer")
            .route("/transactions", web::get().to(get_transactions))
            .route("/transactions/{signature}", web::get().to(get_transaction))
            .route("/nft/{mint}/metadata", web::get().to(get_nft_metadata))
            .route("/subscriptions", web::get().to(get_subscriptions))
            .route("/health", web::get().to(health_check))
    );
}

async fn get_transactions(
    db: web::Data<Arc<Database>>,
    query: web::Query<TransactionQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(50).min(100); // Max 100 results
    let offset = query.offset.unwrap_or(0);
    
    let transactions = if let Some(program_id) = query.program_id.as_ref() {
        db.get_transactions_by_program(program_id, limit, offset).await
            .map_err(|e| {
                eprintln!("Error fetching transactions: {}", e);
                HttpResponse::InternalServerError().json("Internal server error")
            })?
    } else {
        // For now, return empty if no program_id specified
        // You could implement a more general query method
        Vec::new()
    };

    let response: Vec<TransactionResponse> = transactions
        .into_iter()
        .map(|tx| TransactionResponse {
            id: tx.id,
            signature: tx.signature,
            slot: tx.slot,
            block_time: tx.block_time,
            transaction_type: format!("{:?}", tx.transaction_type).to_lowercase(),
            program_id: tx.program_id,
            accounts: tx.accounts,
            status: format!("{:?}", tx.status).to_lowercase(),
            fee: tx.fee,
            created_at: tx.created_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

async fn get_transaction(
    db: web::Data<Arc<Database>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let signature = path.into_inner();
    
    match db.get_transaction_by_signature(&signature).await {
        Ok(Some(transaction)) => {
            let response = TransactionResponse {
                id: transaction.id,
                signature: transaction.signature,
                slot: transaction.slot,
                block_time: transaction.block_time,
                transaction_type: format!("{:?}", transaction.transaction_type).to_lowercase(),
                program_id: transaction.program_id,
                accounts: transaction.accounts,
                status: format!("{:?}", transaction.status).to_lowercase(),
                fee: transaction.fee,
                created_at: transaction.created_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json("Transaction not found")),
        Err(e) => {
            eprintln!("Error fetching transaction: {}", e);
            Ok(HttpResponse::InternalServerError().json("Internal server error"))
        }
    }
}

async fn get_nft_metadata(
    db: web::Data<Arc<Database>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let mint = path.into_inner();
    
    match db.get_nft_metadata_by_mint(&mint).await {
        Ok(Some(metadata)) => {
            let response = NftMetadataResponse {
                id: metadata.id,
                mint: metadata.mint,
                name: metadata.name,
                symbol: metadata.symbol,
                uri: metadata.uri,
                seller_fee_basis_points: metadata.seller_fee_basis_points,
                creators: metadata.creators,
                created_at: metadata.created_at,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json("NFT metadata not found")),
        Err(e) => {
            eprintln!("Error fetching NFT metadata: {}", e);
            Ok(HttpResponse::InternalServerError().json("Internal server error"))
        }
    }
}

async fn get_subscriptions(
    db: web::Data<Arc<Database>>,
) -> Result<HttpResponse> {
    match db.get_subscription_configs().await {
        Ok(subscriptions) => Ok(HttpResponse::Ok().json(subscriptions)),
        Err(e) => {
            eprintln!("Error fetching subscriptions: {}", e);
            Ok(HttpResponse::InternalServerError().json("Internal server error"))
        }
    }
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}
