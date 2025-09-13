use crate::modules::Module;
use actix_web::web;

/// Token module implementation
pub struct TokenModule;

impl Module for TokenModule {
    fn configure_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/token")
                .route("", web::post().to(handlers::create_token))
                .route("/{mint}", web::get().to(handlers::get_token_info)),
        )
        .service(web::scope("/balance").route(
            "/{mint}/{owner}",
            web::get().to(handlers::get_token_balance),
        ))
        .service(web::scope("/mint").route("", web::post().to(handlers::mint_tokens)))
        .service(web::scope("/burn").route("", web::post().to(handlers::burn_tokens)))
        .service(web::scope("/transfer").route("", web::post().to(handlers::transfer_tokens)));
    }

    fn name() -> &'static str {
        "token"
    }

    fn version() -> &'static str {
        "1.0.0"
    }
}

pub mod handlers;
pub mod manager;
pub mod models;
