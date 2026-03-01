use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcAccountSubscribeConfig;
use solana_client::websocket_client::WsClient;
use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta;
use futures::{StreamExt, SinkExt};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct WebSocketService {
    client: Arc<RwLock<Option<WsClient>>>,
    rpc_client: Arc<RpcClient>,
    subscription_config: crate::modules::indexer::models::SubscriptionConfig,
}

impl WebSocketService {
    pub fn new(
        rpc_client: Arc<RpcClient>,
        subscription_config: crate::modules::indexer::models::SubscriptionConfig,
    ) -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            rpc_client,
            subscription_config,
        }
    }

    pub async fn connect(&self) -> Result<()> {
        let ws_client = WsClient::new(&self.subscription_config.websocket_url).await?;
        *self.client.write().await = Some(ws_client);
        Ok(())
    }

    pub async fn subscribe_to_accounts(&self, tx: mpsc::UnboundedSender<EncodedConfirmedTransactionWithStatusMeta>) -> Result<()> {
        let client_guard = self.client.read().await;
        if let Some(ref client) = *client_guard {
            for account_address in &self.subscription_config.account_addresses {
                let pubkey = Pubkey::try_from(account_address)
                    .map_err(|_| anyhow::anyhow!("Invalid account address: {}", account_address))?;
                
                let config = RpcAccountSubscribeConfig {
                    encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                    commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
                };
                
                let (mut subscription, _) = client.account_subscribe(&pubkey, Some(config)).await?;
                
                let tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(account_response) = subscription.next().await {
                        if let Ok(transaction) = self.get_transaction_for_account_change(&account_response.value.pubkey).await {
                            let _ = tx.send(transaction);
                        }
                    }
                });
            }
        }
        Ok(())
    }

    pub async fn subscribe_to_programs(&self, tx: mpsc::UnboundedSender<EncodedConfirmedTransactionWithStatusMeta>) -> Result<()> {
        let client_guard = self.client.read().await;
        if let Some(ref client) = *client_guard {
            for program_id in &self.subscription_config.program_ids {
                let pubkey = Pubkey::try_from(program_id)
                    .map_err(|_| anyhow::anyhow!("Invalid program ID: {}", program_id))?;
                
                let (mut subscription, _) = client.program_subscribe(&pubkey, None).await?;
                
                let tx = tx.clone();
                tokio::spawn(async move {
                    while let Some(program_response) = subscription.next().await {
                        if let Some(signature) = program_response.value.signature {
                            if let Ok(Some(transaction)) = self.rpc_client.get_transaction_with_config(
                                &signature,
                                solana_transaction_status::TransactionConfirmationStatus::Confirmed,
                            ).await {
                                let _ = tx.send(transaction);
                            }
                        }
                    }
                });
            }
        }
        Ok(())
    }

    async fn get_transaction_for_account_change(&self, account_pubkey: &Pubkey) -> Result<EncodedConfirmedTransactionWithStatusMeta> {
        let signatures = self.rpc_client.get_signatures_for_address_with_config(
            account_pubkey,
            solana_client::rpc_config::GetConfirmedSignaturesForAddress2Config {
                limit: Some(1),
                ..Default::default()
            },
        ).await?;
        
        if let Some(signature_info) = signatures.first() {
            let signature = solana_sdk::signature::Signature::from_str(&signature_info.signature)?;
            let transaction = self.rpc_client.get_transaction_with_config(
                &signature,
                solana_transaction_status::TransactionConfirmationStatus::Confirmed,
            ).await?;
            
            Ok(transaction)
        } else {
            Err(anyhow::anyhow!("No transaction found for account change"))
        }
    }
}
