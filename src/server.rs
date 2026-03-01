use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use std::sync::Arc;

use crate::modules::nft::handlers::AppState as NFTAppState;
use crate::modules::nft::manager::NFTManager;
use crate::modules::token::handlers::AppState as TokenAppState;
use crate::modules::token::manager::TokenManager;
use crate::modules::indexer::database::Database;
use crate::modules::Module;

pub async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "solana-token-manager",
        "version": "3.0.0",
        "features": ["create", "mint", "burn", "transfer", "info"]
    })))
}

pub fn create_app(
    token_manager: Arc<TokenManager>,
    nft_manager: Arc<NFTManager>,
    database: Arc<Database>,
) -> impl Fn(&mut web::ServiceConfig) {
    move |cfg: &mut web::ServiceConfig| {
        cfg.app_data(web::Data::new(TokenAppState {
            token_manager: token_manager.clone(),
        }))
        .app_data(web::Data::new(NFTAppState {
            nft_manager: nft_manager.clone(),
        }))
        .app_data(web::Data::new(database.clone()))
        .service(
            web::scope("/api/v1")
                .route("/health", web::get().to(health_check))
                .configure(crate::modules::token::TokenModule::configure_routes)
                .configure(crate::modules::nft::NFTModule::configure_routes)
                .configure(crate::modules::indexer::IndexerModule::configure_routes),
        );
    }
}
