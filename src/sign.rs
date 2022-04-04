use anyhow::{anyhow, Result};
use indicatif::ParallelProgressIterator;
use log::{error, info};
use mpl_token_metadata::{instruction::sign_metadata, state::Metadata, ID as METAPLEX_PROGRAM_ID};
use rayon::prelude::*;
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use std::{
    fs::File,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::decode::get_metadata_pda;
use crate::derive::derive_cmv2_pda;
use crate::limiter::create_rate_limiter;
use crate::parse::{is_only_one_option, parse_keypair};
use crate::snapshot::get_cm_creator_accounts;
use crate::{constants::*, parse::parse_solana_config};

pub fn sign_one(client: &RpcClient, keypair_path: Option<String>, account: String) -> Result<()> {
    let solana_opts = parse_solana_config();
    let creator = parse_keypair(keypair_path, solana_opts);

    let account_pubkey = Pubkey::from_str(&account)?;
    let metadata_pubkey = get_metadata_pda(account_pubkey);

    info!(
        "Signing metadata: {} with creator: {}",
        metadata_pubkey,
        &creator.pubkey()
    );

    let sig = sign(client, &creator, metadata_pubkey)?;
    info!("Tx sig: {}", sig);
    println!("Tx sig: {}", sig);

    Ok(())
}

pub fn sign_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    creator: &Option<String>,
    position: usize,
    v2: bool,
    mint_accounts_file: Option<String>,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let creator_keypair = parse_keypair(keypair_path, solana_opts);

    if !is_only_one_option(creator, &mint_accounts_file) {
        return Err(anyhow!(
            "Must specify exactly one of --candy-machine-id or --mint-data-dir"
        ));
    }

    if let Some(creator) = creator {
        if v2 {
            let creator_pubkey =
                Pubkey::from_str(creator).expect("Failed to parse pubkey from creator!");
            let cmv2_creator = derive_cmv2_pda(&creator_pubkey);
            sign_candy_machine_accounts(
                client,
                &cmv2_creator.to_string(),
                creator_keypair,
                position,
            )?
        } else {
            sign_candy_machine_accounts(client, creator, creator_keypair, position)?
        }
    } else if let Some(mint_accounts_file) = mint_accounts_file {
        let file = File::open(mint_accounts_file)?;
        let mint_accounts: Vec<String> = serde_json::from_reader(&file)?;

        sign_mint_accounts(client, &creator_keypair, mint_accounts)?;
    } else {
        unreachable!();
    }

    Ok(())
}

pub fn sign(client: &RpcClient, creator: &Keypair, metadata_pubkey: Pubkey) -> Result<Signature> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let ix = sign_metadata(METAPLEX_PROGRAM_ID, metadata_pubkey, creator.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&creator.pubkey()),
        &[creator],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}

pub fn sign_mint_accounts(
    client: &RpcClient,
    creator: &Keypair,
    mint_accounts: Vec<String>,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_rate_limiter();

    mint_accounts
        .par_iter()
        .progress()
        .for_each(|mint_account| {
            let mut handle = handle.clone();
            if use_rate_limit {
                handle.wait();
            }

            let account_pubkey = match Pubkey::from_str(mint_account) {
                Ok(pubkey) => pubkey,
                Err(err) => {
                    error!("Invalid public key: {}, error: {}", mint_account, err);
                    return;
                }
            };

            let metadata_pubkey = get_metadata_pda(account_pubkey);

            // Try to sign all accounts, print any errors that crop up.
            match sign(client, creator, metadata_pubkey) {
                Ok(sig) => info!("{}", sig),
                Err(e) => error!("{}", e),
            }
        });

    Ok(())
}

pub fn sign_candy_machine_accounts(
    client: &RpcClient,
    creator: &str,
    signing_creator: Keypair,
    position: usize,
) -> Result<()> {
    let accounts = get_cm_creator_accounts(client, creator, position)?;

    // Only sign accounts that have not been signed yet
    let signed_at_least_one_account = Arc::new(AtomicBool::new(false));

    accounts
        .par_iter()
        .progress()
        .for_each(|(metadata_pubkey, account)| {
            let signed_at_least_one_account = signed_at_least_one_account.clone();
            let metadata: Metadata = match try_from_slice_unchecked(&account.data.clone()) {
                Ok(metadata) => metadata,
                Err(_) => {
                    error!("Account {} has no metadata", metadata_pubkey);
                    return;
                }
            };

            if let Some(creators) = metadata.data.creators {
                // Check whether the specific creator has already signed the account
                for creator in creators {
                    if creator.address == signing_creator.pubkey() && !creator.verified {
                        info!(
                            "Found creator unverified for mint account: {}",
                            metadata.mint
                        );
                        info!("Signing...");

                        let sig = match sign(client, &signing_creator, *metadata_pubkey) {
                            Ok(sig) => sig,
                            Err(e) => {
                                error!("Error signing: {}", e);
                                return;
                            }
                        };

                        info!("{}", sig);

                        signed_at_least_one_account.store(true, Ordering::Relaxed);
                    }
                }
            }
        });

    if !signed_at_least_one_account.load(Ordering::Relaxed) {
        info!("No unverified metadata for this creator and candy machine.");
        println!("No unverified metadata for this creator and candy machine.");
        return Ok(());
    }

    Ok(())
}
