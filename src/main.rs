use actix_web::{App, HttpServer};
use anyhow::Result;
use std::sync::Arc;

mod modules;
mod server;

use modules::nft::manager::NFTManager;
use modules::token::manager::TokenManager;
use crate::modules::indexer::database::Database;
use crate::modules::indexer::background_job::BackgroundIndexer;
use crate::modules::indexer::subscription_manager::SubscriptionManager;
use crate::modules::Module;
use server::create_app;

#[actix_web::main]
async fn main() -> Result<()> {
    // Configuration - you can make these configurable via environment variables
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let payer_keypair_path =
        std::env::var("PAYER_KEYPAIR_PATH").unwrap_or_else(|_| "./payer-keypair.json".to_string());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    println!("🚀 Starting Solana Token Manager Server v3.0...");
    println!("📡 RPC URL: {}", rpc_url);
    println!("🔑 Payer Keypair Path: {}", payer_keypair_path);
    println!("🌐 Port: {}", port);

    // Initialize token manager
    let token_manager = Arc::new(
        TokenManager::new(&rpc_url, &payer_keypair_path)
            .expect("Failed to initialize token manager"),
    );

    // Initialize NFT manager
    let nft_manager = Arc::new(
        NFTManager::new(&rpc_url, &payer_keypair_path).expect("Failed to initialize NFT manager"),
    );

    // Initialize database and indexer
    let database = Arc::new(
        Database::from_env().await.expect("Failed to initialize database"),
    );
    
    // Initialize RPC client
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let rpc_client = Arc::new(solana_client::rpc_client::RpcClient::new(rpc_url));
    
    // Initialize subscription manager
    let (transaction_tx, transaction_rx) = tokio::sync::mpsc::unbounded_channel();
    let subscription_manager = Arc::new(SubscriptionManager::new(
        database.clone(),
        rpc_client.clone(),
        transaction_tx,
    ));
    
    // Initialize existing subscriptions
    if let Err(e) = subscription_manager.initialize_existing_subscriptions().await {
        eprintln!("Failed to initialize existing subscriptions: {}", e);
    }
    
    let mut background_indexer = BackgroundIndexer::new(database.clone(), subscription_manager.clone());
    
    // Start the background indexer
    if let Err(e) = background_indexer.start_with_receiver(transaction_rx).await {
        eprintln!("Failed to start background indexer: {}", e);
    }

    println!("✅ Token manager initialized successfully");
    println!("✅ NFT manager initialized successfully");
    println!("✅ Database initialized successfully");
    println!("✅ Background indexer started successfully");

    println!("📋 Available modules:");
    println!(
        "   🔹 Token Module (v{})",
        crate::modules::token::TokenModule::version()
    );
    println!(
        "   🔹 Token Module (v{})",
        crate::modules::token::TokenModule::name()
    );
    println!("     GET  /api/v1/health");
    println!("     POST /api/v1/token");
    println!("     GET  /api/v1/token/{{mint}}");
    println!("     GET  /api/v1/balance/{{mint}}/{{owner}}");
    println!("     POST /api/v1/mint");
    println!("     POST /api/v1/burn");
    println!("     POST /api/v1/transfer");
    println!(
        "   🔹 NFT Module (v{})",
        crate::modules::nft::NFTModule::version()
    );
    println!("     POST /api/v1/nft");
    println!("     GET  /api/v1/nft/{{mint}}");
    println!("     GET  /api/v1/nft/{{mint}}/owner");
    println!("     POST /api/v1/nft-mint");
    println!("     POST /api/v1/nft-transfer");
    println!("     POST /api/v1/nft-burn");

    println!(
        "   🔹 Indexer Module (v{})",
        crate::modules::indexer::IndexerModule::version()
    );
    println!(
        "   🔹 Indexer Module (v{})",
        crate::modules::indexer::IndexerModule::name()
    );
    println!("     GET  /api/v1/indexer/health");
    println!("     GET  /api/v1/indexer/transactions");
    println!("     GET  /api/v1/indexer/transactions/{{signature}}");
    println!("     GET  /api/v1/indexer/nft/{{mint}}/metadata");
    println!("     GET  /api/v1/indexer/subscriptions");

    // Start HTTP server
    println!("🌐 Starting HTTP server on http://0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new().configure(create_app(
            token_manager.clone(), 
            nft_manager.clone(),
            database.clone(),
            subscription_manager.clone(),
        ))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await?;

    Ok(())
}
