use anyhow::Result;
use metaplex_token_metadata::instruction::update_metadata_accounts;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::{fs::File, str::FromStr};

use crate::constants::*;
use crate::data::NFTData;
use crate::decode::{decode, get_metadata_pda};
use crate::parse::{convert_local_to_remote_data, parse_keypair};

pub fn update_data(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    json_file: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let metadata_account = get_metadata_pda(mint_pubkey);

    let f = File::open(json_file)?;
    let new_data: NFTData = serde_json::from_reader(f)?;

    let update_authority = keypair.pubkey();

    let data = convert_local_to_remote_data(new_data)?;

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        None,
        Some(data),
        None,
    );
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn update_uri(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    new_uri: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);
    let metadata = decode(client, mint_account)?;

    let mut data = metadata.data;
    data.uri = new_uri.to_string();

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        None,
        Some(data),
        None,
    );

    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_primary_sale_happened(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        None,
        None,
        Some(true),
    );
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_update_authority(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    new_update_authority: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();
    let new_update_authority = Pubkey::from_str(new_update_authority)?;

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        Some(new_update_authority),
        None,
        None,
    );
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_update_authority_all(
    client: &RpcClient,
    keypair: &String,
    json_file: &String,
    new_update_authority: &String,
) -> Result<()> {
    let file = File::open(json_file)?;
    let items: Vec<String> = serde_json::from_reader(file)?;

    for item in items.iter() {
        println!("Updating metadata for mint account: {}", item);

        // If someone uses a json list that contains a mint account that has already
        //  been updated this will throw an error. We print that error and continue
        let _ = match set_update_authority(client, keypair, &item, &new_update_authority) {
            Ok(_) => {}
            Err(error) => {
                println!("Error occurred! {}", error)
            }
        };
    }
    Ok(())
}
