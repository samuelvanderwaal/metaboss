use anyhow::{anyhow, Result};
use glob::glob;
use indicatif::ParallelProgressIterator;
use log::{error, info};
use metaplex_token_metadata::{instruction::update_metadata_accounts, state::Data};
use rayon::prelude::*;
use retry::{delay::Fixed, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use std::{fs::File, path::Path, str::FromStr};

use crate::constants::*;
use crate::data::{NFTData, UpdateNFTData, UpdateUriData};
use crate::decode::{decode, get_metadata_pda};
use crate::parse::{convert_local_to_remote_data, parse_keypair};

pub fn update_data_one(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    json_file: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let f = File::open(json_file)?;
    let new_data: NFTData = serde_json::from_reader(f)?;

    let data = convert_local_to_remote_data(new_data)?;

    update_data(client, &keypair, mint_account, data)?;

    Ok(())
}

pub fn update_data_all(client: &RpcClient, keypair: &String, data_dir: &String) -> Result<()> {
    let keypair = parse_keypair(keypair)?;

    let path = Path::new(&data_dir).join("*.json");
    let pattern = path.to_str().ok_or(anyhow!("Invalid directory path"))?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    info!("Updating...");
    println!("Updating...");
    paths.par_iter().progress().for_each(|path| {
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open file: {:?} error: {}", path, e);
                return;
            }
        };

        let update_nft_data: UpdateNFTData = match serde_json::from_reader(f) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Failed to parse JSON data from file: {:?} error: {}",
                    path, e
                );
                return;
            }
        };

        let data = match convert_local_to_remote_data(update_nft_data.nft_data) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Failed to convert local data to remote data: {:?} error: {}",
                    path, e
                );
                return;
            }
        };

        match update_data(client, &keypair, &update_nft_data.mint_account, data) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update data: {:?} error: {}", path, e);
                return;
            }
        }
    });

    // TODO: handle errors in a better way and log instead of print.
    if !errors.is_empty() {
        error!("Failed to read some of the files with the following errors:");
        for error in errors {
            error!("{}", error);
        }
    }

    Ok(())
}

pub fn update_data(
    client: &RpcClient,
    keypair: &Keypair,
    mint_account: &String,
    data: Data,
) -> Result<()> {
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let metadata_account = get_metadata_pda(mint_pubkey);

    let update_authority = keypair.pubkey();

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
        &[keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(Fixed::from_millis(100), || {
        client.send_and_confirm_transaction(&tx)
    });
    let sig = res?;

    info!("Tx sig: {:?}", sig);
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn update_uri_one(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    new_uri: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;

    update_uri(client, &keypair, &mint_account, new_uri)?;

    Ok(())
}

pub fn update_uri_all(client: &RpcClient, keypair: &String, json_file: &String) -> Result<()> {
    let keypair = parse_keypair(keypair)?;

    let f = File::open(json_file)?;
    let update_uris: Vec<UpdateUriData> = serde_json::from_reader(f)?;

    update_uris.par_iter().for_each(|data| {
        match update_uri(client, &keypair, &data.mint_account, &data.new_uri) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update uri: {:?} error: {}", data, e);
                return;
            }
        }
    });

    Ok(())
}

pub fn update_uri(
    client: &RpcClient,
    keypair: &Keypair,
    mint_account: &String,
    new_uri: &String,
) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
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
        &[keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    info!("Tx sig: {:?}", sig);
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
    info!("Tx sig: {:?}", sig);
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
    info!("Tx sig: {:?}", sig);
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

    info!("Setting update_authority...");
    items.par_iter().progress().for_each(|item| {
        info!("Updating metadata for mint account: {}", item);

        // If someone uses a json list that contains a mint account that has already
        //  been updated this will throw an error. We print that error and continue
        let _ = match set_update_authority(client, keypair, &item, &new_update_authority) {
            Ok(_) => {}
            Err(error) => {
                error!("Error occurred! {}", error)
            }
        };
    });

    Ok(())
}
