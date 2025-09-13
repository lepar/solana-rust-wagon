use crate::modules::Module;
use actix_web::web;

/// NFT module implementation
pub struct NFTModule;

impl Module for NFTModule {
    fn configure_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/nft")
                .route("", web::post().to(handlers::create_nft))
                .route("/{mint}", web::get().to(handlers::get_nft_info))
                .route("/{mint}/owner", web::get().to(handlers::get_nft_owner)),
        )
        .service(web::scope("/nft-mint").route("", web::post().to(handlers::mint_nft)))
        .service(web::scope("/nft-transfer").route("", web::post().to(handlers::transfer_nft)))
        .service(web::scope("/nft-burn").route("", web::post().to(handlers::burn_nft)));
    }

    fn name() -> &'static str {
        "nft"
    }

    fn version() -> &'static str {
        "1.0.0"
    }
}

pub mod handlers;
pub mod manager;
pub mod models;
