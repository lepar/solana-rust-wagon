use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};

use crate::modules::indexer::{
    database::Database,
    indexer_service::IndexerService,
    websocket::WebSocketService,
};

pub struct BackgroundIndexer {
    database: Arc<Database>,
    indexer_service: Option<IndexerService>,
    is_running: std::sync::Arc<tokio::sync::RwLock<bool>>,
}

impl BackgroundIndexer {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            database,
            indexer_service: None,
            is_running: std::sync::Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(()); // Already running
        }

        // Initialize RPC client
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
        let rpc_client = Arc::new(solana_client::rpc_client::RpcClient::new(rpc_url));

        // Get subscription configuration
        let subscription_configs = self.database.get_subscription_configs().await?;
        if subscription_configs.is_empty() {
            // Create default subscription config if none exists
            self.create_default_subscription().await?;
            let subscription_configs = self.database.get_subscription_configs().await?;
        }

        // Use first subscription config for now
        let subscription_config = subscription_configs.into_iter().next().unwrap();

        // Initialize WebSocket service
        let websocket_service = Arc::new(WebSocketService::new(rpc_client.clone(), subscription_config));

        // Initialize indexer service
        let (mut indexer_service, _) = IndexerService::new(
            self.database.clone(),
            rpc_client,
            websocket_service,
        );

        // Start the indexer service in a separate task
        let is_running_clone = self.is_running.clone();
        tokio::spawn(async move {
            if let Err(e) = indexer_service.start().await {
                eprintln!("Indexer service error: {}", e);
                let mut running = is_running_clone.write().await;
                *running = false;
            }
        });

        // Start health check task
        self.start_health_check().await?;

        *is_running = true;
        println!("Background indexer started successfully");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        println!("Background indexer stopped");
        Ok(())
    }

    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    async fn create_default_subscription(&self) -> Result<()> {
        let default_config = crate::modules::indexer::models::SubscriptionConfig {
            program_ids: vec![
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(), // SPL Token Program
                "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s".to_string(), // Metaplex Metadata Program
            ],
            account_addresses: vec![],
            websocket_url: "wss://api.mainnet-beta.solana.com".to_string(),
        };

        // Insert default subscription config
        sqlx::query!(
            r#"
            INSERT INTO subscription_configs (name, program_ids, account_addresses, websocket_url)
            VALUES ($1, $2, $3, $4)
            "#,
            "default",
            &default_config.program_ids,
            &default_config.account_addresses,
            default_config.websocket_url
        )
        .execute(self.database.pool())
        .await?;

        Ok(())
    }

    async fn start_health_check(&self) -> Result<()> {
        let is_running = self.is_running.clone();
        let database = self.database.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute
            
            loop {
                interval.tick().await;
                
                let running = *is_running.read().await;
                if !running {
                    break;
                }

                // Perform health check
                if let Err(e) = database.get_subscription_configs().await {
                    eprintln!("Health check failed: {}", e);
                }
            }
        });

        Ok(())
    }

    pub async fn get_status(&self) -> IndexerStatus {
        IndexerStatus {
            is_running: *self.is_running.read().await,
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct IndexerStatus {
    pub is_running: bool,
    pub uptime: u64,
}
