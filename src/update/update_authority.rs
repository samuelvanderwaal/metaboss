use anyhow::Result;
use indicatif::ParallelProgressIterator;
use log::{error, info};
use mpl_token_metadata::instruction::update_metadata_accounts_v2;
use rayon::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::{fs::File, str::FromStr};

use crate::decode::get_metadata_pda;
use crate::limiter::create_default_rate_limiter;
use crate::parse::parse_keypair;
use crate::{constants::*, parse::parse_solana_config};

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
