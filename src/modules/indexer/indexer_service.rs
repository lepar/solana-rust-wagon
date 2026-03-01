use anyhow::Result;
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiInstruction};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::modules::indexer::{
    database::Database,
    models::{IndexedTransaction, TokenTransfer, NftMetadata, TransactionType, TransactionStatus},
    websocket::WebSocketService,
};

pub struct IndexerService {
    database: Arc<Database>,
    rpc_client: Arc<RpcClient>,
    websocket_service: Arc<WebSocketService>,
    transaction_rx: mpsc::UnboundedReceiver<EncodedConfirmedTransactionWithStatusMeta>,
}

impl IndexerService {
    pub fn new(
        database: Arc<Database>,
        rpc_client: Arc<RpcClient>,
        websocket_service: Arc<WebSocketService>,
    ) -> (Self, mpsc::UnboundedSender<EncodedConfirmedTransactionWithStatusMeta>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let service = Self {
            database,
            rpc_client,
            websocket_service,
            transaction_rx: rx,
        };
        
        (service, tx)
    }

    pub async fn start(&mut self) -> Result<()> {
        // Connect to WebSocket
        self.websocket_service.connect().await?;
        
        // Start subscriptions
        let (tx, _) = mpsc::unbounded_channel();
        self.websocket_service.subscribe_to_accounts(tx.clone()).await?;
        self.websocket_service.subscribe_to_programs(tx).await?;
        
        // Start processing transactions
        self.process_transactions().await
    }

    async fn process_transactions(&mut self) -> Result<()> {
        while let Some(transaction) = self.transaction_rx.recv().await {
            if let Err(e) = self.process_single_transaction(transaction).await {
                eprintln!("Error processing transaction: {}", e);
            }
        }
        Ok(())
    }

    async fn process_single_transaction(&self, transaction: EncodedConfirmedTransactionWithStatusMeta) -> Result<()> {
        let transaction_id = Uuid::new_v4();
        let signature = transaction.transaction.signatures.first()
            .ok_or_else(|| anyhow::anyhow!("Transaction has no signature"))?;
        
        let signature_str = bs58::encode(signature).into_string();
        
        // Check if transaction already exists
        if self.database.get_transaction_by_signature(&signature_str).await?.is_some() {
            return Ok(());
        }

        let slot = transaction.slot;
        let block_time = transaction.block_time.map(|time| chrono::DateTime::from_timestamp(time, 0).unwrap_or_else(Utc::now));
        
        let mut indexed_transactions = Vec::new();
        let mut token_transfers = Vec::new();
        let mut nft_metadata_updates = Vec::new();

        // Process each instruction in the transaction
        if let Some(meta) = &transaction.transaction.meta {
            for (index, instruction) in transaction.transaction.message.instructions.iter().enumerate() {
                if let Ok(parsed_instruction) = self.parse_instruction(instruction, &transaction.transaction.message, index).await {
                    let transaction_type = self.determine_transaction_type(&parsed_instruction);
                    
                    let indexed_tx = IndexedTransaction {
                        id: transaction_id,
                        signature: signature_str.clone(),
                        slot,
                        block_time,
                        transaction_type,
                        program_id: parsed_instruction.program_id,
                        accounts: parsed_instruction.accounts,
                        data: parsed_instruction.data,
                        status: if meta.err.is_some() { TransactionStatus::Failure } else { TransactionStatus::Success },
                        fee: meta.fee,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    
                    indexed_transactions.push(indexed_tx);
                    
                    // Extract specific data based on transaction type
                    match transaction_type {
                        TransactionType::TokenTransfer => {
                            if let Some(transfer) = self.extract_token_transfer(&parsed_instruction).await? {
                                token_transfers.push(transfer);
                            }
                        }
                        TransactionType::NftMint | TransactionType::NftMetadataUpdate => {
                            if let Some(metadata) = self.extract_nft_metadata(&parsed_instruction).await? {
                                nft_metadata_updates.push(metadata);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // Store in database
        for indexed_tx in indexed_transactions {
            self.database.store_transaction(&indexed_tx).await?;
        }
        
        for transfer in token_transfers {
            self.database.store_token_transfer(&transfer).await?;
        }
        
        for metadata in nft_metadata_updates {
            self.database.store_nft_metadata(&metadata).await?;
        }

        Ok(())
    }

    async fn parse_instruction(&self, instruction: &UiInstruction, message: &solana_transaction_status::EncodedConfirmedTransaction, index: usize) -> Result<ParsedInstruction> {
        // This is a simplified parsing - you'll need to implement proper instruction parsing
        // based on the specific programs you want to support
        
        let program_id_index = instruction.program_id_index as usize;
        let program_id = if program_id_index < message.message.account_keys.len() {
            &message.message.account_keys[program_id_index]
        } else {
            return Err(anyhow::anyhow!("Invalid program ID index"));
        };

        let accounts: Vec<String> = instruction.accounts.iter()
            .map(|&account_index| {
                if account_index as usize < message.message.account_keys.len() {
                    message.message.account_keys[account_index as usize].clone()
                } else {
                    String::new()
                }
            })
            .collect();

        Ok(ParsedInstruction {
            program_id: program_id.clone(),
            accounts,
            data: instruction.data.clone(),
        })
    }

    fn determine_transaction_type(&self, instruction: &ParsedInstruction) -> TransactionType {
        // Known program IDs
        let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        let metadata_program = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
        
        match instruction.program_id.as_str() {
            token_program => {
                // Simple heuristic based on instruction data
                if instruction.data.len() >= 1 {
                    match instruction.data.as_bytes()[0] {
                        1 | 2 => TransactionType::TokenTransfer,
                        3 => TransactionType::TokenMint,
                        4 => TransactionType::TokenBurn,
                        _ => TransactionType::Unknown,
                    }
                } else {
                    TransactionType::Unknown
                }
            }
            metadata_program => {
                if instruction.data.len() >= 1 {
                    match instruction.data.as_bytes()[0] {
                        0 => TransactionType::NftMint,
                        1 => TransactionType::NftMetadataUpdate,
                        _ => TransactionType::Unknown,
                    }
                } else {
                    TransactionType::Unknown
                }
            }
            _ => TransactionType::Unknown,
        }
    }

    async fn extract_token_transfer(&self, instruction: &ParsedInstruction) -> Result<Option<TokenTransfer>> {
        // This is a simplified implementation - you'll need to properly decode the instruction data
        // based on the SPL Token program format
        
        if instruction.accounts.len() >= 3 {
            Ok(Some(TokenTransfer {
                id: Uuid::new_v4(),
                transaction_id: Uuid::new_v4(), // This should be passed from the caller
                mint: instruction.accounts.get(0).cloned().unwrap_or_default(),
                from_account: instruction.accounts.get(1).cloned().unwrap_or_default(),
                to_account: instruction.accounts.get(2).cloned().unwrap_or_default(),
                amount: "0".to_string(), // Parse from instruction data
                decimals: 0, // Get from mint account
                created_at: Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn extract_nft_metadata(&self, instruction: &ParsedInstruction) -> Result<Option<NftMetadata>> {
        // This is a simplified implementation - you'll need to properly decode the instruction data
        // based on the Metaplex Metadata program format
        
        if instruction.accounts.len() >= 3 {
            Ok(Some(NftMetadata {
                id: Uuid::new_v4(),
                transaction_id: Uuid::new_v4(), // This should be passed from the caller
                mint: instruction.accounts.get(1).cloned().unwrap_or_default(),
                name: None, // Parse from instruction data
                symbol: None, // Parse from instruction data
                uri: None, // Parse from instruction data
                seller_fee_basis_points: None, // Parse from instruction data
                creators: None, // Parse from instruction data
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
struct ParsedInstruction {
    program_id: String,
    accounts: Vec<String>,
    data: String,
}
