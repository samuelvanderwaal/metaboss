use crate::decode::get_metadata_pda;
use crate::limiter::create_default_rate_limiter;
use crate::parse::parse_keypair;
use crate::{constants::*, parse::parse_solana_config};
use anyhow::Result;
use indicatif::ParallelProgressIterator;
use log::{error, info};
use mpl_token_metadata::instruction::update_metadata_accounts_v2;
use rayon::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::{fs::File, str::FromStr};

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
