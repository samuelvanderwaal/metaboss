use anyhow::Result;
use mpl_token_metadata::{
    id as metadata_program_id,
    instruction::{
        approve_collection_authority, revoke_collection_authority, set_and_verify_collection,
        unverify_collection, verify_collection,
    },
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;

use crate::parse::parse_keypair;
use crate::{
    derive::{derive_collection_authority_record, derive_edition_pda, derive_metadata_pda},
    parse::parse_solana_config,
};

fn send_and_confirm_transaction(
    client: &RpcClient,
    keypair: Keypair,
    instructions: &[Instruction],
) -> Result<String> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
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
    Ok(sig.to_string())
}

pub fn set_and_verify_nft_collection(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    nft_auth: String,
    is_delegate_present: bool,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_metadata = derive_metadata_pda(&Pubkey::from_str(&nft_mint)?);
    let nft_update_authority = Pubkey::from_str(&nft_auth)?;
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_metadata = derive_metadata_pda(&collection_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(derive_collection_authority_record(&collection_pubkey, &keypair.pubkey()).0),
        false => None,
    };

    let set_and_verify_ix = set_and_verify_collection(
        metadata_program_id(),
        nft_metadata,
        keypair.pubkey(),
        keypair.pubkey(),
        nft_update_authority,
        collection_pubkey,
        collection_metadata,
        collection_edition_pubkey,
        collection_authority_record,
    );

    send_and_confirm_transaction(client, keypair, &[set_and_verify_ix])?;

    Ok(())
}

pub fn unverify_nft_collection(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_metadata = derive_metadata_pda(&Pubkey::from_str(&nft_mint)?);
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_metadata = derive_metadata_pda(&collection_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(derive_collection_authority_record(&collection_pubkey, &keypair.pubkey()).0),
        false => None,
    };

    let unverify_collection_ix = unverify_collection(
        metadata_program_id(),
        nft_metadata,
        keypair.pubkey(),
        collection_pubkey,
        collection_metadata,
        collection_edition_pubkey,
        collection_authority_record,
    );

    send_and_confirm_transaction(client, keypair, &[unverify_collection_ix])?;

    Ok(())
}

pub fn verify_nft_collection(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_metadata = derive_metadata_pda(&Pubkey::from_str(&nft_mint)?);
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_metadata = derive_metadata_pda(&collection_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(derive_collection_authority_record(&collection_pubkey, &keypair.pubkey()).0),
        false => None,
    };

    let verify_collection_ix = verify_collection(
        metadata_program_id(),
        nft_metadata,
        keypair.pubkey(),
        keypair.pubkey(),
        collection_pubkey,
        collection_metadata,
        collection_edition_pubkey,
        collection_authority_record,
    );

    send_and_confirm_transaction(client, keypair, &[verify_collection_ix])?;

    Ok(())
}

pub fn approve_delegate(
    client: &RpcClient,
    keypair_path: Option<String>,
    collection_mint: String,
    delegate_authority: String,
) -> Result<()> {
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let (collection_authority_record, _bump) =
        derive_collection_authority_record(&collection_pubkey, &delegate_pubkey);

    let metadata = derive_metadata_pda(&collection_pubkey);

    let approve_collection_auth_ix = approve_collection_authority(
        metadata_program_id(),
        collection_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        keypair.pubkey(),
        metadata,
        collection_pubkey,
    );

    send_and_confirm_transaction(client, keypair, &[approve_collection_auth_ix])?;

    Ok(())
}

pub fn revoke_delegate(
    client: &RpcClient,
    keypair_path: Option<String>,
    collection_mint: String,
    delegate_authority: String,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let (collection_authority_record, _bump) =
        derive_collection_authority_record(&collection_pubkey, &delegate_pubkey);

    let metadata = derive_metadata_pda(&collection_pubkey);

    let revoke_collection_auth_ix = revoke_collection_authority(
        metadata_program_id(),
        collection_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        metadata,
        collection_pubkey,
    );

    send_and_confirm_transaction(client, keypair, &[revoke_collection_auth_ix])?;

    Ok(())
}
