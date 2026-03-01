use actix_web::web;

/// Trait that all modules must implement to provide their routes
pub trait Module {
    /// Configure the module's routes
    fn configure_routes(cfg: &mut web::ServiceConfig);

    /// Get the module's name
    fn name() -> &'static str;

    /// Get the module's version
    fn version() -> &'static str;
}

pub mod nft;
pub mod token;
pub mod indexer;
