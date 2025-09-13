use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    signer::keypair::read_keypair_file,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{burn, mint_to, transfer},
    solana_program::program_pack::Pack,
    state::Mint,
};

use super::models::*;

pub struct TokenManager {
    client: std::sync::Arc<RpcClient>,
    payer: Keypair,
}

impl TokenManager {
    pub fn new(rpc_url: &str, payer_keypair_path: &str) -> Result<Self> {
        let client = std::sync::Arc::new(RpcClient::new(rpc_url.to_string()));
        let payer = read_keypair_file(payer_keypair_path)
            .map_err(|e| anyhow!("Failed to read keypair file: {}", e))?;

        Ok(Self { client, payer })
    }

    pub async fn create_token(&self, request: CreateTokenRequest) -> Result<TokenInfo> {
        use solana_sdk::system_instruction;
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

        // Initialize mint
        let initialize_mint_ix = initialize_mint(
            &spl_token::id(),
            &mint_pubkey,
            &self.payer.pubkey(), // mint authority
            None,                 // freeze authority (optional)
            request.decimals,
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

        Ok(TokenInfo {
            mint: mint_pubkey.to_string(),
            decimals: request.decimals,
            supply: 0, // Initial supply is 0, tokens need to be minted separately
        })
    }

    pub async fn mint_tokens(&self, request: MintRequest) -> Result<Signature> {
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

        let mint_to_ix = mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &associated_token_account,
            &self.payer.pubkey(), // mint authority
            &[],
            request.amount,
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

    pub async fn burn_tokens(&self, request: BurnRequest) -> Result<Signature> {
        let mint_pubkey = request.mint.parse::<Pubkey>()?;
        let from_pubkey = request.from.parse::<Pubkey>()?;

        let associated_token_account = get_associated_token_address(&from_pubkey, &mint_pubkey);

        let burn_ix = burn(
            &spl_token::id(),
            &associated_token_account,
            &mint_pubkey,
            &from_pubkey,
            &[],
            request.amount,
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

    pub async fn transfer_tokens(&self, request: TransferRequest) -> Result<Signature> {
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

        let transfer_ix = transfer(
            &spl_token::id(),
            &from_associated_token_account,
            &to_associated_token_account,
            &from_pubkey,
            &[],
            request.amount,
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

    pub async fn get_token_info(&self, mint: &str) -> Result<TokenInfo> {
        let mint_pubkey = mint.parse::<Pubkey>()?;

        let account = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let mint_pubkey = mint_pubkey.clone();
            move || client.get_account(&mint_pubkey)
        })
        .await??;
        let mint_data = Mint::unpack(&account.data)?;

        Ok(TokenInfo {
            mint: mint_pubkey.to_string(),
            decimals: mint_data.decimals,
            supply: mint_data.supply,
        })
    }

    pub async fn get_token_balance(&self, mint: &str, owner: &str) -> Result<u64> {
        let mint_pubkey = mint.parse::<Pubkey>()?;
        let owner_pubkey = owner.parse::<Pubkey>()?;

        // Get associated token address
        let associated_token_account = get_associated_token_address(&owner_pubkey, &mint_pubkey);

        // Get the token account
        let account = tokio::task::spawn_blocking({
            let client = self.client.clone();
            let associated_token_account = associated_token_account.clone();
            move || client.get_account(&associated_token_account)
        })
        .await??;

        if account.data.is_empty() {
            return Ok(0); // Account doesn't exist, balance is 0
        }

        // Unpack the token account data
        let token_account = spl_token::state::Account::unpack(&account.data)?;
        Ok(token_account.amount)
    }
}
