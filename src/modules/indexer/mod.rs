use actix_web::web;
use crate::modules::Module;

pub mod models;
pub mod database;
pub mod websocket;
pub mod indexer_service;
pub mod routes;
pub mod background_job;
pub mod subscription_manager;

pub struct IndexerModule;

impl Module for IndexerModule {
    fn configure_routes(cfg: &mut web::ServiceConfig) {
        routes::configure(cfg);
    }

    fn name() -> &'static str {
        "indexer"
    }

    fn version() -> &'static str {
        "0.1.0"
    }
}
