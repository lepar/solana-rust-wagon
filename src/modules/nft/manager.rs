use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    signer::keypair::read_keypair_file,
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{burn, transfer},
    solana_program::program_pack::Pack,
    state::Mint,
};

use super::models::*;

pub struct NFTManager {
    client: std::sync::Arc<RpcClient>,
    payer: Keypair,
}

impl NFTManager {
    pub fn new(rpc_url: &str, payer_keypair_path: &str) -> Result<Self> {
        let client = std::sync::Arc::new(RpcClient::new(rpc_url.to_string()));
        let payer = read_keypair_file(payer_keypair_path)
            .map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;

        Ok(Self { client, payer })
    }

    pub async fn create_nft(&self, request: CreateNFTRequest) -> Result<NFTInfo> {
        use spl_token::instruction::initialize_mint;

        // Create mint keypair
        let mint_keypair = Keypair::new();
        let mint_pubkey = mint_keypair.pubkey();

        // Get rent-exempt minimum balance
        let rent = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)
        })
        .await??;

        // Create mint account
        let create_mint_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &mint_pubkey,
            rent,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        );

        // Initialize mint with 0 decimals (NFTs have 0 decimals)
        let initialize_mint_ix = initialize_mint(
            &spl_token::id(),
            &mint_pubkey,
            &self.payer.pubkey(), // mint authority
            None,                 // freeze authority (optional)
            0,                    // decimals (NFTs always have 0 decimals)
        )?;

        let mut transaction = Transaction::new_with_payer(
            &[create_mint_account_ix, initialize_mint_ix],
            Some(&self.payer.pubkey()),
        );

        let recent_blockhash = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_latest_blockhash()
        })
        .await??;

        transaction.sign(&[&self.payer, &mint_keypair], recent_blockhash);

        let _signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.send_and_confirm_transaction(&transaction)
        })
        .await??;

        Ok(NFTInfo {
            mint: mint_pubkey.to_string(),
            name: request.name,
            symbol: request.symbol,
            uri: request.uri,
            seller_fee_basis_points: request.seller_fee_basis_points,
            creators: request.creators,
        })
    }

    pub async fn mint_nft(&self, request: MintNFTRequest) -> Result<Signature> {
        use spl_token::instruction::mint_to;

        let mint_pubkey = request.mint.parse::<Pubkey>()?;
        let to_pubkey = request.to.parse::<Pubkey>()?;

        // Verify the mint account exists and is valid
        let mint_account = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let mint_pubkey = mint_pubkey.clone();
            move || client.get_account(&mint_pubkey)
        })
        .await??;

        if mint_account.data.is_empty() {
            return Err(anyhow!("Mint account does not exist or is not initialized"));
        }

        // Get associated token address
        let associated_token_account = get_associated_token_address(&to_pubkey, &mint_pubkey);

        // Check if the associated token account exists, create if not
        let account_exists = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let associated_token_account = associated_token_account.clone();
            move || match client.get_account(&associated_token_account) {
                Ok(account) => !account.data.is_empty(),
                Err(_) => false,
            }
        })
        .await;

        let mut instructions = vec![];

        if !account_exists.unwrap_or(false) {
            let create_ata_ix = create_associated_token_account(
                &self.payer.pubkey(),
                &to_pubkey,
                &mint_pubkey,
                &spl_token::id(),
            );
            instructions.push(create_ata_ix);
        }

        // Mint exactly 1 token (NFTs are always quantity 1)
        let mint_to_ix = mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &associated_token_account,
            &self.payer.pubkey(), // mint authority
            &[],
            1, // NFTs are always quantity 1
        )?;

        instructions.push(mint_to_ix);

        let mut transaction =
            Transaction::new_with_payer(&instructions, Some(&self.payer.pubkey()));

        let recent_blockhash = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_latest_blockhash()
        })
        .await??;

        transaction.sign(&[&self.payer], recent_blockhash);

        let signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.send_transaction(&transaction)
        })
        .await??;

        Ok(signature)
    }

    pub async fn transfer_nft(&self, request: TransferNFTRequest) -> Result<Signature> {
        let mint_pubkey = request.mint.parse::<Pubkey>()?;
        let from_pubkey = request.from.parse::<Pubkey>()?;
        let to_pubkey = request.to.parse::<Pubkey>()?;

        let from_associated_token_account =
            get_associated_token_address(&from_pubkey, &mint_pubkey);

        let to_associated_token_account = get_associated_token_address(&to_pubkey, &mint_pubkey);

        // Check if the destination associated token account exists, create if not
        let account_info = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let to_associated_token_account = to_associated_token_account.clone();
            move || client.get_account(&to_associated_token_account)
        })
        .await;
        let mut instructions = vec![];

        if account_info.is_err() {
            let create_ata_ix = create_associated_token_account(
                &self.payer.pubkey(),
                &to_pubkey,
                &mint_pubkey,
                &spl_token::id(),
            );
            instructions.push(create_ata_ix);
        }

        // Transfer exactly 1 token (NFTs are always quantity 1)
        let transfer_ix = transfer(
            &spl_token::id(),
            &from_associated_token_account,
            &to_associated_token_account,
            &from_pubkey,
            &[],
            1, // NFTs are always quantity 1
        )?;

        instructions.push(transfer_ix);

        let mut transaction = Transaction::new_with_payer(&instructions, Some(&from_pubkey));

        let recent_blockhash = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_latest_blockhash()
        })
        .await??;

        transaction.sign(&[&self.payer], recent_blockhash);

        let signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.send_and_confirm_transaction(&transaction)
        })
        .await??;

        Ok(signature)
    }

    pub async fn burn_nft(&self, request: BurnNFTRequest) -> Result<Signature> {
        let mint_pubkey = request.mint.parse::<Pubkey>()?;
        let from_pubkey = request.from.parse::<Pubkey>()?;

        let associated_token_account = get_associated_token_address(&from_pubkey, &mint_pubkey);

        // Burn exactly 1 token (NFTs are always quantity 1)
        let burn_ix = burn(
            &spl_token::id(),
            &associated_token_account,
            &mint_pubkey,
            &from_pubkey,
            &[],
            1, // NFTs are always quantity 1
        )?;

        let mut transaction = Transaction::new_with_payer(&[burn_ix], Some(&from_pubkey));

        let recent_blockhash = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_latest_blockhash()
        })
        .await??;

        transaction.sign(&[&self.payer], recent_blockhash);

        let signature = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.send_and_confirm_transaction(&transaction)
        })
        .await??;

        Ok(signature)
    }

    pub async fn get_nft_info(&self, mint: &str) -> Result<NFTInfo> {
        let mint_pubkey = mint.parse::<Pubkey>()?;

        let account = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let mint_pubkey = mint_pubkey.clone();
            move || client.get_account(&mint_pubkey)
        })
        .await??;
        let _mint_data = Mint::unpack(&account.data)?;

        // For now, return basic info. In a real implementation, you'd fetch metadata from URI
        Ok(NFTInfo {
            mint: mint_pubkey.to_string(),
            name: "NFT".to_string(),
            symbol: "NFT".to_string(),
            uri: "".to_string(),
            seller_fee_basis_points: 0,
            creators: vec![],
        })
    }

    pub async fn get_nft_owner(&self, mint: &str) -> Result<Option<String>> {
        let mint_pubkey = mint
            .parse::<Pubkey>()
            .map_err(|e| anyhow!("Invalid mint address: {}", e))?;

        // Get the largest token accounts for this mint (for NFTs, this should be the owner)
        let token_accounts = tokio::task::spawn_blocking({
            let client = self.client.clone();
            move || client.get_token_largest_accounts(&mint_pubkey)
        })
        .await??;

        // For NFTs, we expect only one account with a balance > 0
        for account in token_accounts {
            if account.amount.ui_amount.unwrap_or(0.0) > 0.0 {
                // Get the account data to extract the owner
                let account_address = account
                    .address
                    .parse::<Pubkey>()
                    .map_err(|e| anyhow!("Invalid account address: {}", e))?;

                let account_data = tokio::task::spawn_blocking({
                    let client = self.client.clone();
                    move || client.get_account(&account_address)
                })
                .await??;

                // Parse the token account data to get the owner
                // Token account data structure: [mint(32), owner(32), amount(8), ...]
                if account_data.data.len() >= 64 {
                    let owner_bytes = &account_data.data[32..64];
                    let owner_pubkey = Pubkey::try_from(owner_bytes)
                        .map_err(|e| anyhow!("Failed to parse owner pubkey: {}", e))?;
                    return Ok(Some(owner_pubkey.to_string()));
                }
            }
        }

        // No owner found (NFT might be burned or not minted)
        Ok(None)
    }
}
