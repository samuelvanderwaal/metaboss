use anyhow::Result;
use mpl_token_metadata::{
    id,
    instruction::{approve_collection_authority, revoke_collection_authority},
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::str::FromStr;

use crate::derive::{derive_collection_authority_record, derive_metadata_pda};
use crate::parse::parse_keypair;

pub fn approve_delegate(
    client: &RpcClient,
    keypair: String,
    collection_mint: String,
    delegate_authority: String,
) -> Result<()> {
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let keypair = parse_keypair(&keypair)?;
    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let (collection_authority_record, _bump) =
        derive_collection_authority_record(&collection_pubkey, &delegate_pubkey);

    let metadata = derive_metadata_pda(&collection_pubkey);

    let ix = approve_collection_authority(
        id(),
        collection_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        keypair.pubkey(),
        metadata,
        collection_pubkey,
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    println!("TxId: {}", sig);

    Ok(())
}

pub fn revoke_delegate(
    client: &RpcClient,
    keypair: String,
    collection_mint: String,
    delegate_authority: String,
) -> Result<()> {
    let keypair = parse_keypair(&keypair)?;
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let (collection_authority_record, _bump) =
        derive_collection_authority_record(&collection_pubkey, &delegate_pubkey);

    let metadata = derive_metadata_pda(&collection_pubkey);

    let ix = revoke_collection_authority(
        id(),
        collection_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        metadata,
        collection_pubkey,
    );

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    println!("TxId: {}", sig);

    Ok(())
}
