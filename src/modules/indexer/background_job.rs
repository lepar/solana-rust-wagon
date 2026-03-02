use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration};

use crate::modules::indexer::database::Database;
use crate::modules::indexer::background_job::BackgroundIndexer;
use crate::modules::indexer::subscription_manager::{SubscriptionManager, ActiveSubscription};
use crate::modules::Module;

pub struct BackgroundIndexer {
    database: Arc<Database>,
    subscription_manager: Arc<SubscriptionManager>,
    is_running: std::sync::Arc<tokio::sync::RwLock<bool>>,
}

impl BackgroundIndexer {
    pub fn new(database: Arc<Database>, subscription_manager: Arc<SubscriptionManager>) -> Self {
        Self {
            database,
            subscription_manager,
            is_running: std::sync::Arc::new(tokio::sync::RwLock::new(false)),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(()); // Already running
        }

        // Start health check task
        self.start_health_check().await?;

        *is_running = true;
        println!("Background indexer started successfully");
        Ok(())
    }

    pub async fn start_with_receiver(&mut self, mut transaction_rx: tokio::sync::mpsc::UnboundedReceiver<solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta>) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(()); // Already running
        }

        // Start health check task
        self.start_health_check().await?;

        // Start transaction processing task
        let is_running_clone = self.is_running.clone();
        let database = self.database.clone();
        tokio::spawn(async move {
            while let Some(transaction) = transaction_rx.recv().await {
                let running = *is_running_clone.read().await;
                if !running {
                    break;
                }

                if let Err(e) = Self::process_transaction(&database, transaction).await {
                    eprintln!("Error processing transaction: {}", e);
                }
            }
        });

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
        let active_subscriptions = self.subscription_manager.get_active_subscriptions().await;
        IndexerStatus {
            is_running: *self.is_running.read().await,
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            active_subscriptions: active_subscriptions.len(),
        }
    }

    async fn process_transaction(database: &Database, transaction: solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta) -> Result<()> {
        // Simplified transaction processing - you can expand this with the full logic
        // from indexer_service.rs if needed
        let transaction_id = uuid::Uuid::new_v4();
        let signature = transaction.transaction.signatures.first()
            .ok_or_else(|| anyhow::anyhow!("Transaction has no signature"))?;
        
        let signature_str = bs58::encode(signature).into_string();
        
        // Check if transaction already exists
        if database.get_transaction_by_signature(&signature_str).await?.is_some() {
            return Ok(());
        }

        // For now, just store a basic transaction record
        // You can expand this with full parsing logic
        println!("Processing transaction: {}", signature_str);
        
        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct IndexerStatus {
    pub is_running: bool,
    pub uptime: u64,
    pub active_subscriptions: usize,
}
