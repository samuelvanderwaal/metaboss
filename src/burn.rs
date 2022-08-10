use anyhow::Result as AnyResult;
use async_trait::async_trait;
use borsh::BorshDeserialize;
use mpl_token_metadata::{id, instruction::burn_nft, state::Metadata};
use retry::{delay::Exponential, retry};
pub use solana_client::{
    nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::{str::FromStr, sync::Arc};

use crate::{
    cache::{Action, BatchActionArgs, RunActionArgs},
    derive::{derive_edition_pda, derive_metadata_pda},
    errors::ActionError,
    parse::{parse_keypair, parse_solana_config},
};

pub async fn burn_one(
    client: RpcClient,
    keypair: Option<String>,
    mint_address: String,
) -> AnyResult<()> {
    let mint_pubkey = Pubkey::from_str(&mint_address)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair, solana_opts);

    let client = Arc::new(client);
    let keypair = Arc::new(keypair);

    let args = BurnArgs {
        client,
        keypair,
        mint_pubkey,
    };

    let sig = burn(args).await?;

    println!("TxId: {}", sig);

    Ok(())
}

pub struct BurnAll {}

pub struct BurnAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct BurnArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_pubkey: Pubkey,
}

#[async_trait]
impl Action for BurnAll {
    fn name() -> &'static str {
        "burn-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        let mint_pubkey = Pubkey::from_str(&args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

        let _sig = burn(BurnArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_pubkey,
        })
        .await
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

        Ok(())
    }
}

pub async fn burn_all(args: BurnAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list: args.mint_list,
        cache_file: args.cache_file,
        new_value: String::new(),
        batch_size: args.batch_size,
        retries: args.retries,
    };
    BurnAll::run(args).await?;

    Ok(())
}

// client: &RpcClient,
// signer: &Keypair,
// owner_pubkey: Pubkey,
// mint_pubkey: Pubkey,

pub async fn burn(args: BurnArgs) -> AnyResult<Signature> {
    let assoc = get_associated_token_address(&args.keypair.pubkey(), &args.mint_pubkey);
    let spl_token_program_id = spl_token::id();
    let metadata_pubkey = derive_metadata_pda(&args.mint_pubkey);
    let master_edition = derive_edition_pda(&args.mint_pubkey);

    let md_account = args.client.get_account_data(&metadata_pubkey)?;
    let metadata = Metadata::deserialize(&mut md_account.as_slice())?;

    // Is it a verified collection item?
    let collection_md = if let Some(collection) = metadata.collection {
        if collection.verified {
            let collection_metadata_pubkey = derive_metadata_pda(&collection.key);
            Some(collection_metadata_pubkey)
        } else {
            None
        }
    } else {
        None
    };

    let burn_ix = burn_nft(
        id(),
        metadata_pubkey,
        args.keypair.pubkey(),
        args.mint_pubkey,
        assoc,
        master_edition,
        spl_token_program_id,
        collection_md,
    );

    let instructions = vec![burn_ix];

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&args.keypair.pubkey()),
        &[&*args.keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}
