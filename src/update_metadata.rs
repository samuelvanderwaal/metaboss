use anyhow::{anyhow, Result};
use glob::glob;
use indicatif::ParallelProgressIterator;
use log::{error, info, warn};
use mpl_token_metadata::{instruction::update_metadata_accounts_v2, state::DataV2};
use rayon::prelude::*;
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use std::{
    cmp,
    fs::File,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::data::{NFTData, UpdateNFTData, UpdateUriData};
use crate::decode::{decode, get_metadata_pda};
use crate::limiter::create_default_rate_limiter;
use crate::parse::{convert_local_to_remote_data, parse_cli_creators, parse_keypair};
use crate::{constants::*, parse::parse_solana_config};

pub fn update_seller_fee_basis_points_one(
    client: &RpcClient,
    keypair: Option<String>,
    mint_account: &str,
    new_seller_fee_basis_points: &u16,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let parsed_keypair = parse_keypair(keypair, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_seller_fee_basis_points = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_seller_fee_basis_points.creators,
        seller_fee_basis_points: new_seller_fee_basis_points.to_owned(),
        name: data_with_old_seller_fee_basis_points.name,
        symbol: data_with_old_seller_fee_basis_points.symbol,
        uri: data_with_old_seller_fee_basis_points.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &parsed_keypair, mint_account, new_data)?;
    Ok(())
}

pub fn update_name_one(
    client: &RpcClient,
    keypair: Option<String>,
    mint_account: &str,
    new_name: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let parsed_keypair = parse_keypair(keypair, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_name = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_name.creators,
        seller_fee_basis_points: data_with_old_name.seller_fee_basis_points,
        name: new_name.to_owned(),
        symbol: data_with_old_name.symbol,
        uri: data_with_old_name.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &parsed_keypair, mint_account, new_data)?;
    Ok(())
}

pub fn update_symbol_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_symbol: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_symbol = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_symbol.creators,
        seller_fee_basis_points: data_with_old_symbol.seller_fee_basis_points,
        name: data_with_old_symbol.name,
        symbol: new_symbol.to_owned(),
        uri: data_with_old_symbol.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &keypair, mint_account, new_data)?;
    Ok(())
}

pub fn update_creator_by_position(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_creators: &str,
    should_append: bool,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_creators = old_md.data;
    let parsed_creators = parse_cli_creators(new_creators.to_string(), should_append)?;

    let new_creators = if let Some(mut old_creators) = data_with_old_creators.creators {
        if !should_append {
            parsed_creators
        } else {
            let remaining_space = 5 - old_creators.len();
            warn!(
                "Appending {} new creators with old creators with shares of 0",
                parsed_creators.len()
            );
            let end_index = cmp::min(parsed_creators.len(), remaining_space);
            old_creators.append(&mut parsed_creators[0..end_index].to_vec());
            old_creators
        }
    } else {
        parsed_creators
    };

    let shares = new_creators.iter().fold(0, |acc, c| acc + c.share);
    if shares != 100 {
        return Err(anyhow!("Creators shares must sum to 100!"));
    }

    let new_data = DataV2 {
        creators: Some(new_creators),
        seller_fee_basis_points: data_with_old_creators.seller_fee_basis_points,
        name: data_with_old_creators.name,
        symbol: data_with_old_creators.symbol,
        uri: data_with_old_creators.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &keypair, mint_account, new_data)?;
    Ok(())
}

pub fn update_data_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    json_file: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(client, mint_account)?;

    let f = File::open(json_file)?;
    let new_data: NFTData = serde_json::from_reader(f)?;

    let data = convert_local_to_remote_data(new_data)?;

    let data_v2 = DataV2 {
        creators: data.creators,
        seller_fee_basis_points: data.seller_fee_basis_points,
        name: data.name,
        symbol: data.symbol,
        uri: data.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &keypair, mint_account, data_v2)?;

    Ok(())
}

pub fn update_data_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    data_dir: &str,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let path = Path::new(&data_dir).join("*.json");
    let pattern = path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid directory path"))?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    let failed_mints: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    info!("Updating...");
    println!("Updating...");
    paths.par_iter().progress().for_each(|path| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        let failed_mints = failed_mints.clone();
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

        let old_md = match decode(client, &update_nft_data.mint_account) {
            Ok(md) => md,
            Err(e) => {
                error!(
                    "Failed to decode mint account: {} error: {}",
                    update_nft_data.mint_account, e
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

        let data_v2 = DataV2 {
            creators: data.creators,
            seller_fee_basis_points: data.seller_fee_basis_points,
            name: data.name,
            symbol: data.symbol,
            uri: data.uri,
            collection: old_md.collection,
            uses: old_md.uses,
        };

        match update_data(client, &keypair, &update_nft_data.mint_account, data_v2) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update data: {:?} error: {}", path, e);
                failed_mints
                    .lock()
                    .unwrap()
                    .push(update_nft_data.mint_account);
            }
        }
    });

    if !errors.is_empty() {
        error!("Failed to read some of the files with the following errors:");
        for error in errors {
            error!("{}", error);
        }
    }

    if !failed_mints.lock().unwrap().is_empty() {
        error!("Failed to update the following mints:");
        for mint in failed_mints.lock().unwrap().iter() {
            error!("{}", mint);
        }
    }

    Ok(())
}

pub fn update_data(
    client: &RpcClient,
    keypair: &Keypair,
    mint_account: &str,
    data: DataV2,
) -> Result<()> {
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let metadata_account = get_metadata_pda(mint_pubkey);

    let update_authority = keypair.pubkey();

    let ix = update_metadata_accounts_v2(
        program_id,
        metadata_account,
        update_authority,
        None,
        Some(data),
        None,
        None,
    );
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    info!("Tx sig: {:?}", sig);
    println!("Mint: {:?}, Tx sig: {:?}", mint_account, sig);

    Ok(())
}

pub fn update_uri_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_uri: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    update_uri(client, &keypair, mint_account, new_uri)?;

    Ok(())
}

pub fn update_uri_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    json_file: &str,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let f = File::open(json_file)?;
    let update_uris: Vec<UpdateUriData> = serde_json::from_reader(f)?;

    update_uris.par_iter().for_each(|data| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        match update_uri(client, &keypair, &data.mint_account, &data.new_uri) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update uri: {:?} error: {}", data, e);
            }
        }
    });

    Ok(())
}

pub fn update_uri(
    client: &RpcClient,
    keypair: &Keypair,
    mint_account: &str,
    new_uri: &str,
) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);
    let metadata = decode(client, mint_account)?;

    let mut data = metadata.data;
    if data.uri.trim_matches(char::from(0)) != new_uri.trim_matches(char::from(0)) {
        data.uri = new_uri.to_string();

        let data_v2 = DataV2 {
            name: data.name,
            symbol: data.symbol,
            uri: data.uri,
            seller_fee_basis_points: data.seller_fee_basis_points,
            creators: data.creators,
            collection: metadata.collection,
            uses: metadata.uses,
        };

        let ix = update_metadata_accounts_v2(
            program_id,
            metadata_account,
            update_authority,
            None,
            Some(data_v2),
            None,
            None,
        );

        let recent_blockhash = client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&update_authority),
            &[keypair],
            recent_blockhash,
        );

        let sig = client.send_and_confirm_transaction(&tx)?;
        info!("Tx sig: {:?}", sig);
        println!("Tx sig: {:?}", sig);
    } else {
        println!("URI is the same.");
    }

    Ok(())
}

pub fn set_primary_sale_happened(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts_v2(
        program_id,
        metadata_account,
        update_authority,
        None,
        None,
        Some(true),
        None,
    );
    let recent_blockhash = client.get_latest_blockhash()?;
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
    keypair_path: Option<String>,
    mint_account: &str,
    new_update_authority: &str,
    keypair_payer_path: Option<String>,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path.clone(), solana_opts);

    let solana_opts = parse_solana_config();
    let keypair_payer = parse_keypair(keypair_payer_path.or(keypair_path), solana_opts);

    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();
    let new_update_authority = Pubkey::from_str(new_update_authority)?;

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts_v2(
        program_id,
        metadata_account,
        update_authority,
        Some(new_update_authority),
        None,
        None,
        None,
    );
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair_payer.pubkey()),
        &[&keypair, &keypair_payer],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    info!("Tx sig: {:?}", sig);
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_update_authority_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    json_file: &str,
    new_update_authority: &str,
    keypair_payer_path: Option<String>,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    let file = File::open(json_file)?;
    let items: Vec<String> = serde_json::from_reader(file)?;

    info!("Setting update_authority...");
    items.par_iter().progress().for_each(|item| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        // If someone uses a json list that contains a mint account that has already
        //  been updated this will throw an error. We print that error and continue
        match set_update_authority(
            client,
            keypair_path.clone(),
            item,
            new_update_authority,
            keypair_payer_path.clone(),
        ) {
            Ok(_) => {}
            Err(error) => {
                error!("Error occurred! {}", error)
            }
        };
    });

    Ok(())
}

pub fn set_immutable(
    client: &RpcClient,
    keypair_path: Option<String>,
    account: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_account = Pubkey::from_str(account)?;

    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_account);

    let ix = update_metadata_accounts_v2(
        program_id,
        metadata_account,
        update_authority,
        None,
        None,
        None,
        Some(false),
    );
    let recent_blockhash = client.get_latest_blockhash()?;
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

pub fn set_immutable_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    json_file: &str,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    let file = File::open(json_file)?;
    let items: Vec<String> = serde_json::from_reader(file)?;

    info!("Setting immutable...");
    items.par_iter().progress().for_each(|item| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        // If someone uses a json list that contains a mint account that has already
        //  been updated this will throw an error. We print that error and continue
        match set_immutable(client, keypair_path.clone(), item) {
            Ok(_) => {}
            Err(error) => {
                error!("Error occurred! {}", error)
            }
        };
    });

    Ok(())
}
