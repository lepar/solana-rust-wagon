use actix_web::{App, HttpServer};
use anyhow::Result;
use std::sync::Arc;

mod modules;
mod server;

use modules::nft::manager::NFTManager;
use modules::token::manager::TokenManager;
use modules::Module;
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

    println!("✅ Token manager initialized successfully");
    println!("✅ NFT manager initialized successfully");

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

    // Start HTTP server
    println!("🌐 Starting HTTP server on http://0.0.0.0:{}", port);

    HttpServer::new(move || {
        App::new().configure(create_app(token_manager.clone(), nft_manager.clone()))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await?;

    Ok(())
}
