use anyhow::{anyhow, Result as AnyResult};
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
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    str::FromStr,
    sync::Arc,
};

use crate::{
    derive::{derive_collection_authority_record, derive_edition_pda, derive_metadata_pda},
    errors::MigrateError,
    parse::parse_solana_config,
    spinner::create_spinner,
    utils::{async_send_and_confirm_transaction, send_and_confirm_transaction},
};
use crate::{parse::parse_keypair, snapshot::get_mint_accounts};

pub const OPEN_FILES_LIMIT: usize = 1024;

pub struct MigrateArgs {
    pub client: RpcClient,
    pub async_client: AsyncRpcClient,
    pub keypair: Option<String>,
    pub mint_address: String,
    pub candy_machine_id: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub retries: u8,
}

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

    pub fn update_errors(&mut self, errors: Vec<Result<(), MigrateError>>) {
        let errors = errors.iter().map(|r| r.as_ref()).map(Result::unwrap_err);

        // Clear out old errors.
        self.0.clear();

        for error in errors {
            match error {
                MigrateError::MigrationFailed(mint_address, _) => {
                    let item = CacheItem {
                        error: Some(error.to_string()),
                    };

                    self.0.insert(mint_address.to_string(), item);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheItem {
    pub error: Option<String>,
}

pub fn set_and_verify_nft_collection(
    client: RpcClient,
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

    send_and_confirm_transaction(&client, keypair, &[set_and_verify_ix])?;

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
        &Pubkey::from_str(&nft_mint).unwrap_or_else(|_| panic!("invalid pubkey: {:?}", nft_mint)),
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
    client: RpcClient,
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

    send_and_confirm_transaction(&client, keypair, &[unverify_collection_ix])?;

    Ok(())
}

pub fn verify_nft_collection(
    client: RpcClient,
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

    send_and_confirm_transaction(&client, keypair, &[verify_collection_ix])?;

    Ok(())
}

pub fn approve_delegate(
    client: RpcClient,
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

    send_and_confirm_transaction(&client, keypair, &[approve_collection_auth_ix])?;

    Ok(())
}

pub fn revoke_delegate(
    client: RpcClient,
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

    send_and_confirm_transaction(&client, keypair, &[revoke_collection_auth_ix])?;

    Ok(())
}

pub async fn migrate_collection(args: MigrateArgs) -> AnyResult<()> {
    if args.candy_machine_id.is_some() && args.mint_list.is_some() {
        return Err(anyhow!(
            "Please specify either a candy machine id or an mint_list file, but not both."
        ));
    }

    if args.cache_file.is_some() && (args.candy_machine_id.is_some() || args.mint_list.is_some()) {
        return Err(anyhow!(
            "Cannot use cache option with either a candy machine id or an mint_list file."
        ));
    }

    let mut cache_file_name = "metaboss-cache-migrate-collection.json".to_string();
    let mut cache = MigrateCache::new();

    let solana_opts = parse_solana_config();
    let keypair = Arc::new(parse_keypair(args.keypair, solana_opts));

    let mut mint_accounts = if let Some(candy_machine_id) = args.candy_machine_id {
        println!("Using candy machine id to fetch mint list. . .");
        get_mint_accounts(&args.client, &Some(candy_machine_id), 0, None, true)?
    } else if let Some(mint_list) = args.mint_list {
        let f = File::open(mint_list)?;
        serde_json::from_reader(f)?
    } else if let Some(cache_path) = args.cache_file {
        println!("Retrying items from cache file. . .");
        cache_file_name = cache_path;

        let f = File::open(&cache_file_name)?;
        let cache: MigrateCache = serde_json::from_reader(f)?;
        cache.0.keys().map(|k| k.to_string()).collect()
    } else {
        return Err(anyhow!(
            "Please specify either a candy machine id or an mint_list file."
        ));
    };

    let f = if !Path::new(&cache_file_name).exists() {
        File::create(&cache_file_name)?
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(&cache_file_name)?
    };

    let async_client = Arc::new(args.async_client);

    let mut counter = 0u8;

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
                    args.mint_address.clone(),
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
        if !migrate_failed.is_empty() && counter < args.retries {
            counter += 1;
            println!(
                "{}/{} migrations failed. Retrying. . .",
                &migrate_failed.len(),
                migrate_tasks_len
            );
            cache.update_errors(migrate_failed);
            mint_accounts = cache.0.keys().map(|m| m.to_string()).collect();
        } else if migrate_failed.is_empty() {
            // None failed so we exit the loop.
            println!("All items successfully migrated!");
            break;
        } else {
            println!("Reached max retries. Writing remaining items to cache.");
            cache.write(f)?;
            break;
        }
    }

    Ok(())
}
