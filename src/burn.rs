use anyhow::Result;
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
use std::str::FromStr;

use crate::derive::{derive_edition_pda, derive_metadata_pda};
use crate::parse::parse_keypair;
use crate::parse::parse_solana_config;

pub fn burn_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_address: String,
) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(&mint_address)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);
    let owner_pubkey = keypair.pubkey();

    let sig = burn(client, &keypair, owner_pubkey, mint_pubkey)?;

    println!("TxId: {}", sig);

    Ok(())
}

pub fn burn(
    client: &RpcClient,
    signer: &Keypair,
    owner_pubkey: Pubkey,
    mint_pubkey: Pubkey,
) -> Result<Signature> {
    let assoc = get_associated_token_address(&owner_pubkey, &mint_pubkey);
    let spl_token_program_id = spl_token::id();
    let metadata_pubkey = derive_metadata_pda(&mint_pubkey);
    let master_edition = derive_edition_pda(&mint_pubkey);

    let md_account = client.get_account_data(&metadata_pubkey)?;
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
        owner_pubkey,
        mint_pubkey,
        assoc,
        master_edition,
        spl_token_program_id,
        collection_md,
    );

    let instructions = vec![burn_ix];

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&signer.pubkey()),
        &[signer],
        recent_blockhash,
    );

    println!("Sending tx...");
    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}
