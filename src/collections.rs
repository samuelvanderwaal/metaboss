use anyhow::{anyhow, Result as AnyResult};
use dialoguer::Confirm;
use indexmap::IndexMap;
use log::info;
use mpl_token_metadata::{
    id as metadata_program_id,
    instruction::{
        approve_collection_authority, revoke_collection_authority, set_and_verify_collection,
        unverify_collection, verify_collection,
    },
};
use serde::{Deserialize, Serialize};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::{fs::File, io::Write, str::FromStr, sync::Arc};

use crate::{
    derive::{derive_collection_authority_record, derive_edition_pda, derive_metadata_pda},
    errors::MigrateError,
    parse::{is_only_one_option, parse_solana_config},
    spinner::create_spinner,
    utils::{async_send_and_confirm_transaction, send_and_confirm_transaction},
};
use crate::{parse::parse_keypair, snapshot::get_mint_accounts};

pub const OPEN_FILES_LIMIT: usize = 1024;

#[derive(Debug, Deserialize, Serialize)]
pub struct MigrateCache(IndexMap<String, CacheItem>);
pub type MigrateResults = Vec<Result<(), MigrateError>>;

impl Default for MigrateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrateCache {
    pub fn new() -> Self {
        MigrateCache(IndexMap::new())
    }

    pub fn write<W: Write>(self, writer: W) -> AnyResult<()> {
        serde_json::to_writer(writer, &self)?;
        Ok(())
    }

    pub fn write_errors<W: Write>(self, writer: W) -> AnyResult<()> {
        let errors = self
            .0
            .iter()
            .filter(|(_, v)| !v.successful)
            .collect::<IndexMap<_, _>>();
        serde_json::to_writer(writer, &errors)?;
        Ok(())
    }

    pub fn update_errors(&mut self, errors: Vec<Result<(), MigrateError>>) {
        let errors = errors.iter().map(|r| r.as_ref()).map(Result::unwrap_err);

        // This is really verbose; surely there's a better way.
        for error in errors {
            match error {
                MigrateError::MigrationFailed(mint_address, _) => {
                    *self.0.get_mut(mint_address).unwrap() = CacheItem {
                        successful: false,
                        error: Some(error.to_string()),
                    };
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CacheItem {
    pub successful: bool,
    pub error: Option<String>,
}

pub fn set_and_verify_nft_collection(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    nft_auth: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
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

async fn set_and_verify(
    async_client: Arc<AsyncRpcClient>,
    authority_keypair: Arc<Keypair>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> Result<(), MigrateError> {
    let nft_metadata_pubkey = derive_metadata_pda(
        &Pubkey::from_str(&nft_mint).expect(&format!("invalid pubkey: {:?}", nft_mint)),
    );
    let collection_mint_pubkey = Pubkey::from_str(&collection_mint).unwrap();
    let collection_metadata_pubkey = derive_metadata_pda(&collection_mint_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_mint_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(
            derive_collection_authority_record(
                &collection_mint_pubkey,
                &authority_keypair.pubkey(),
            )
            .0,
        ),
        false => None,
    };

    let set_and_verify_ix = set_and_verify_collection(
        metadata_program_id(),
        nft_metadata_pubkey,
        authority_keypair.pubkey(),
        authority_keypair.pubkey(),
        authority_keypair.pubkey(),
        collection_mint_pubkey,
        collection_metadata_pubkey,
        collection_edition_pubkey,
        collection_authority_record,
    );

    async_send_and_confirm_transaction(async_client, authority_keypair, &[set_and_verify_ix])
        .await
        .map_err(|e| MigrateError::MigrationFailed(nft_mint, e.to_string()))?;

    Ok(())
}

pub fn unverify_nft_collection(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
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
) -> AnyResult<()> {
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
) -> AnyResult<()> {
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
) -> AnyResult<()> {
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

pub async fn migrate_collection(
    client: &RpcClient,
    async_client: AsyncRpcClient,
    keypair_path: Option<String>,
    collection_mint: String,
    candy_machine_id: Option<String>,
    mint_list: Option<String>,
) -> AnyResult<()> {
    if !is_only_one_option(&candy_machine_id, &mint_list) {
        return Err(anyhow!(
            "Please specify either a candy machine id or an mint_list file, but not both."
        ));
    }

    let solana_opts = parse_solana_config();
    let keypair = Arc::new(parse_keypair(keypair_path, solana_opts));

    let mut mint_accounts = if let Some(candy_machine_id) = candy_machine_id {
        println!("Using candy machine id to fetch mint list. . .");
        get_mint_accounts(&client, &Some(candy_machine_id), 0, None, true)?
    } else if let Some(mint_list) = mint_list {
        let f = File::open(mint_list)?;
        serde_json::from_reader(f)?
    } else {
        return Err(anyhow!(
            "Please specify either a candy machine id or an mint_list file."
        ));
    };

    let cache_file_name = "metaboss-cache-migrate-collection.json";

    // Create a temp cache file to track failures.
    let f = File::create(cache_file_name)?;

    // Mint addresses success starts out as false and we will update on success.
    let mut cache = MigrateCache::new();
    for mint in &mint_accounts {
        cache.0.insert(
            mint.clone(),
            CacheItem {
                successful: false,
                error: None,
            },
        );
    }

    let async_client = Arc::new(async_client);

    // Loop over migrate process so we can retry repeatedly until the user exits.
    loop {
        let remaining_mints = mint_accounts.clone();

        info!("Sending network requests...");
        let spinner = create_spinner("Sending network requests....");
        // Create a vector of futures to execute.
        let migrate_tasks: Vec<_> = remaining_mints
            .into_iter()
            .map(|mint_account| {
                tokio::spawn(set_and_verify(
                    async_client.clone(),
                    keypair.clone(),
                    mint_account,
                    collection_mint.clone(),
                    false,
                ))
            })
            .collect();
        spinner.finish();

        let migrate_tasks_len = migrate_tasks.len();

        // Wait for all the tasks to resolve and push the results to our results vector
        let mut migrate_results = Vec::new();
        let spinner = create_spinner("Awaiting results...");
        for task in migrate_tasks {
            migrate_results.push(task.await.unwrap());
        }
        spinner.finish();

        // Partition migration results.
        let (_migrate_successful, migrate_failed): (MigrateResults, MigrateResults) =
            migrate_results.into_iter().partition(Result::is_ok);

        // If some of the migrations failed, ask user if they wish to retry and the loop starts again.
        // Otherwise, break out of the loop and write the cache to disk.
        if !migrate_failed.is_empty() {
            let msg = format!(
                "{}/{} migrations failed. Do you want to retry these ones?",
                &migrate_failed.len(),
                migrate_tasks_len
            );
            if Confirm::new().with_prompt(msg).interact()? {
                mint_accounts = cache
                    .0
                    .keys()
                    .filter(|&k| !cache.0[k].successful)
                    .map(|m| m.to_string())
                    .collect();
                continue;
            } else {
                // We have failures but the user has decided to stop so we log failures to the cache file.
                cache.update_errors(migrate_failed);

                println!("Writing cache to file...{}", cache_file_name);
                cache.write_errors(f)?;
                break;
            }
        } else {
            // None failed so we exit the loop.
            println!("All items successfully migrated!");
            break;
        }
    }

    Ok(())
}
