use anyhow::{anyhow, Result};
use metaplex_token_metadata::{
    instruction::sign_metadata, state::Metadata, ID as METAPLEX_PROGRAM_ID,
};
use rayon::prelude::*;
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
    sync::{Arc, Mutex},
};

use crate::decode::get_metadata_pda;
use crate::parse::{is_only_one_option, parse_keypair};
use crate::snapshot::get_cm_creator_accounts;

pub fn sign_one(client: &RpcClient, keypair: String, account: String) -> Result<()> {
    let creator = parse_keypair(&keypair)?;
    let account_pubkey = Pubkey::from_str(&account)?;

    let metadata_pubkey = get_metadata_pda(account_pubkey);

    let sig = sign(client, &creator, metadata_pubkey)?;
    println!("{}", sig);

    Ok(())
}

pub fn sign_all(
    client: &RpcClient,
    keypair: &String,
    candy_machine_id: Option<String>,
    mint_accounts_file: Option<String>,
) -> Result<()> {
    let creator = parse_keypair(keypair)?;

    if !is_only_one_option(&candy_machine_id, &mint_accounts_file) {
        return Err(anyhow!(
            "Must specify exactly one of --candy-machine-id or --mint-data-dir"
        ));
    }

    if let Some(candy_machine_id) = candy_machine_id {
        sign_candy_machine_accounts(client, &candy_machine_id, creator)?;
    } else if let Some(mint_accounts_file) = mint_accounts_file {
        let file = File::open(mint_accounts_file)?;
        let mint_accounts: Vec<String> = serde_json::from_reader(&file)?;

        sign_mint_accounts(client, &creator, mint_accounts)?;
    } else {
        unreachable!();
    }

    Ok(())
}

pub fn sign(client: &RpcClient, creator: &Keypair, metadata_pubkey: Pubkey) -> Result<Signature> {
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let ix = sign_metadata(METAPLEX_PROGRAM_ID, metadata_pubkey, creator.pubkey());
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&creator.pubkey()),
        &[creator],
        recent_blockhash,
    );
    let sig = client.send_and_confirm_transaction(&tx)?;
    Ok(sig)
}

pub fn sign_mint_accounts(
    client: &RpcClient,
    creator: &Keypair,
    mint_accounts: Vec<String>,
) -> Result<()> {
    mint_accounts.par_iter().for_each(|mint_account| {
        let account_pubkey = match Pubkey::from_str(&mint_account) {
            Ok(pubkey) => pubkey,
            Err(err) => {
                eprintln!("Invalid public key: {}, error: {}", mint_account, err);
                return;
            }
        };

        let metadata_pubkey = get_metadata_pda(account_pubkey);

        // Try to sign all accounts, print any errors that crop up.
        match sign(client, &creator, metadata_pubkey) {
            Ok(sig) => println!("{}", sig),
            Err(e) => println!("{}", e),
        }
    });

    Ok(())
}

pub fn sign_candy_machine_accounts(
    client: &RpcClient,
    candy_machine_id: &String,
    signing_creator: Keypair,
) -> Result<()> {
    let accounts = get_cm_creator_accounts(client, candy_machine_id)?;

    // Only sign accounts that have not been signed yet
    let signed_at_least_one_account = Arc::new(Mutex::new(false));

    accounts.par_iter().for_each(|(metadata_pubkey, account)| {
        let signed_at_least_one_account = signed_at_least_one_account.clone();
        let metadata: Metadata = match try_from_slice_unchecked(&account.data.clone()) {
            Ok(metadata) => metadata,
            Err(_) => {
                eprintln!("Account {} has no metadata", metadata_pubkey);
                return;
            }
        };

        if let Some(creators) = metadata.data.creators {
            // Check whether the specific creator has already signed the account
            for creator in creators {
                if creator.address == signing_creator.pubkey() && !creator.verified {
                    println!(
                        "Found creator unverified for mint account: {}",
                        metadata.mint
                    );
                    println!("Signing...");

                    let sig = match sign(client, &signing_creator, *metadata_pubkey) {
                        Ok(sig) => sig,
                        Err(e) => {
                            eprintln!("Error signing: {}", e);
                            return;
                        }
                    };

                    println!("{}", sig);

                    *signed_at_least_one_account.lock().unwrap() = true;
                }
            }
        } else {
            // No creators for that token, nothing to sign.
            return;
        }
    });

    if !*signed_at_least_one_account.lock().unwrap() {
        println!("No unverified metadata for this creator and candy machine.");
        return Ok(());
    }

    Ok(())
}
