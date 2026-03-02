use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::modules::indexer::{
    database::Database,
    websocket::WebSocketService,
    indexer_service::IndexerService,
    models::SubscriptionConfig,
};

#[derive(Debug, Clone)]
pub struct ActiveSubscription {
    pub id: Uuid,
    pub config: SubscriptionConfig,
    pub is_running: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct SubscriptionManager {
    database: Arc<Database>,
    active_subscriptions: Arc<RwLock<HashMap<Uuid, ActiveSubscription>>>,
    rpc_client: Arc<solana_client::rpc_client::RpcClient>,
    transaction_tx: mpsc::UnboundedSender<solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta>,
}

impl SubscriptionManager {
    pub fn new(
        database: Arc<Database>,
        rpc_client: Arc<solana_client::rpc_client::RpcClient>,
        transaction_tx: mpsc::UnboundedSender<solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta>,
    ) -> Self {
        Self {
            database,
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            rpc_client,
            transaction_tx,
        }
    }

    pub async fn add_subscription(&self, config: SubscriptionConfig) -> Result<Uuid> {
        let subscription_id = Uuid::new_v4();
        
        // Save to database
        sqlx::query!(
            r#"
            INSERT INTO subscription_configs (id, name, program_ids, account_addresses, websocket_url, is_active)
            VALUES ($1, $2, $3, $4, $5, true)
            "#,
            subscription_id,
            format!("subscription_{}", subscription_id),
            &config.program_ids,
            &config.account_addresses,
            config.websocket_url
        )
        .execute(self.database.pool())
        .await?;

        // Create active subscription record
        let active_subscription = ActiveSubscription {
            id: subscription_id,
            config: config.clone(),
            is_running: false,
            created_at: chrono::Utc::now(),
        };

        // Add to active subscriptions
        {
            let mut subscriptions = self.active_subscriptions.write().await;
            subscriptions.insert(subscription_id, active_subscription);
        }

        // Spawn the subscription service
        self.spawn_subscription_service(subscription_id, config).await?;

        Ok(subscription_id)
    }

    pub async fn remove_subscription(&self, subscription_id: Uuid) -> Result<()> {
        // Mark as inactive in database
        sqlx::query!(
            r#"
            UPDATE subscription_configs 
            SET is_active = false 
            WHERE id = $1
            "#,
            subscription_id
        )
        .execute(self.database.pool())
        .await?;

        // Remove from active subscriptions
        {
            let mut subscriptions = self.active_subscriptions.write().await;
            subscriptions.remove(&subscription_id);
        }

        println!("Subscription {} removed", subscription_id);
        Ok(())
    }

    pub async fn get_active_subscriptions(&self) -> Vec<ActiveSubscription> {
        let subscriptions = self.active_subscriptions.read().await;
        subscriptions.values().cloned().collect()
    }

    pub async fn get_subscription_by_id(&self, subscription_id: Uuid) -> Option<ActiveSubscription> {
        let subscriptions = self.active_subscriptions.read().await;
        subscriptions.get(&subscription_id).cloned()
    }

    async fn spawn_subscription_service(&self, subscription_id: Uuid, config: SubscriptionConfig) -> Result<()> {
        let websocket_service = Arc::new(WebSocketService::new(self.rpc_client.clone(), config.clone()));
        let tx = self.transaction_tx.clone();
        let active_subscriptions = self.active_subscriptions.clone();

        tokio::spawn(async move {
            println!("Starting subscription service for {}", subscription_id);
            
            // Connect to WebSocket
            if let Err(e) = websocket_service.connect().await {
                eprintln!("Failed to connect WebSocket for subscription {}: {}", subscription_id, e);
                return;
            }

            // Mark as running
            {
                let mut subscriptions = active_subscriptions.write().await;
                if let Some(subscription) = subscriptions.get_mut(&subscription_id) {
                    subscription.is_running = true;
                }
            }

            // Start subscriptions
            if let Err(e) = websocket_service.subscribe_to_accounts(tx.clone()).await {
                eprintln!("Failed to subscribe to accounts for {}: {}", subscription_id, e);
            }

            if let Err(e) = websocket_service.subscribe_to_programs(tx).await {
                eprintln!("Failed to subscribe to programs for {}: {}", subscription_id, e);
            }

            println!("Subscription service {} started successfully", subscription_id);
        });

        Ok(())
    }

    pub async fn initialize_existing_subscriptions(&self) -> Result<()> {
        let configs = self.database.get_subscription_configs().await?;
        
        for config in configs {
            let subscription_id = Uuid::new_v4(); // Generate new ID for this session
            
            let active_subscription = ActiveSubscription {
                id: subscription_id,
                config: config.clone(),
                is_running: false,
                created_at: chrono::Utc::now(),
            };

            {
                let mut subscriptions = self.active_subscriptions.write().await;
                subscriptions.insert(subscription_id, active_subscription);
            }

            self.spawn_subscription_service(subscription_id, config).await?;
        }

        Ok(())
    }
}
