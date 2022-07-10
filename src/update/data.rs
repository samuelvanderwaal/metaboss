use anyhow::{anyhow, Result};
use glob::glob;
use indicatif::ParallelProgressIterator;
use log::{error, info};
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
    fs::File,
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::data::{NFTData, UpdateNFTData};
use crate::decode::{decode, get_metadata_pda};
use crate::limiter::create_default_rate_limiter;
use crate::parse::{convert_local_to_remote_data, parse_keypair};
use crate::{constants::*, parse::parse_solana_config};

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

    info!("Mint: {:?}, Tx sig: {:?}", mint_account, sig);

    Ok(())
}
