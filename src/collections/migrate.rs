use super::*;

use crate::constants::NANO_SECONDS_IN_SECOND;
use crate::limiter::create_rate_limiter_with_capacity;
use crate::spinner::create_progress_bar;
use crate::{derive::derive_metadata_pda, errors::MigrateError, parse::parse_solana_config};
use crate::{parse::parse_keypair, snapshot::get_mint_accounts};
use metaboss_lib::{
    data::Priority,
    unverify::{unverify_collection_ix, UnverifyCollectionArgs},
    update::{update_asset_ix, UpdateAssetArgs, V1UpdateArgs},
    verify::{verify_collection_ix, VerifyCollectionArgs},
};
use mpl_token_metadata::types::CollectionToggle;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
use std::ops::{Deref, DerefMut};

pub struct MigrateArgs {
    pub client: RpcClient,
    pub async_client: AsyncRpcClient,
    pub keypair: Option<String>,
    pub mint_address: String,
    pub candy_machine_id: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub retries: u8,
    pub rate_limit: usize,
    pub output_file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MigrateCache(IndexMap<String, CacheItem>);
pub type MigrateResults = Vec<Result<(), MigrateError>>;

impl Default for MigrateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for MigrateCache {
    type Target = IndexMap<String, CacheItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MigrateCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl MigrateCache {
    pub fn new() -> Self {
        MigrateCache(IndexMap::new())
    }

    pub fn write<W: Write>(&mut self, writer: W) -> AnyResult<()> {
        self.sort_unstable_keys();
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    pub fn update_errors(&mut self, errors: Vec<MigrateError>) {
        self.clear();

        for error in errors {
            match error {
                MigrateError::MigrationFailed(ref mint_address, _) => {
                    let item = CacheItem {
                        error: Some(error.to_string()),
                    };

                    self.insert(mint_address.to_string(), item);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheItem {
    pub error: Option<String>,
}

async fn set_and_verify(
    client: Arc<RpcClient>,
    authority_keypair: Arc<Keypair>,
    nft_mint: String,
    collection_mint: String,
    is_delegate: bool,
) -> Result<Signature, MigrateError> {
    let nft_metadata_pubkey = derive_metadata_pda(
        &Pubkey::from_str(&nft_mint).unwrap_or_else(|_| panic!("invalid pubkey: {nft_mint:?}")),
    );
    let collection_mint_pubkey = Pubkey::from_str(&collection_mint).unwrap();

    let mut instructions = Vec::new();

    // Whether or not it is a sized collection, if it is a verified collection item,
    // we need to unverify it before we can set it.
    let nft_md_account = client
        .get_account_data(&nft_metadata_pubkey)
        .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;
    let nft_metadata = <Metadata as BorshDeserialize>::deserialize(&mut nft_md_account.as_slice())
        .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;

    if let Some(current_collection) = nft_metadata.collection {
        if current_collection.verified {
            let current_collection_mint = current_collection.key;

            let unverify_args = UnverifyCollectionArgs::V1 {
                authority: &authority_keypair,
                mint: nft_mint.clone(),
                collection_mint: current_collection_mint,
                is_delegate,
            };

            // This instruction handles both the case where the collection NFT exists and the case where it doesn't.
            let ix = unverify_collection_ix(&client, unverify_args)
                .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;
            instructions.push(ix);
        }
    }

    // Add update instruction to set the collection.
    // Token Metadata UpdateArgs enum.
    let update_args = V1UpdateArgs {
        collection: CollectionToggle::Set(MdCollection {
            key: collection_mint_pubkey,
            verified: false,
        }),
        ..Default::default()
    };

    let update_ix = update_asset_ix(
        &client,
        UpdateAssetArgs::V1 {
            authority: &authority_keypair,
            mint: nft_mint.clone(),
            payer: None,
            token: None::<String>,
            delegate_record: None::<String>, // Not supported yet in update.
            update_args,
            priority: Priority::None,
        },
    )
    .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;

    instructions.push(update_ix);

    // Add verify instruction to verify the collection.
    let verify_args = VerifyCollectionArgs::V1 {
        authority: &authority_keypair,
        mint: nft_mint.clone(),
        collection_mint,
        is_delegate,
    };

    // This instruction handles both the case where the collection NFT exists and the case where it doesn't.
    let verify_ix = verify_collection_ix(&client, verify_args)
        .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;

    instructions.push(verify_ix);

    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;

    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&authority_keypair.pubkey()),
        &[&*authority_keypair],
        recent_blockhash,
    );

    let sig = client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| MigrateError::MigrationFailed(nft_mint.clone(), e.to_string()))?;

    Ok(sig)
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

    // Default name, if we don't get an output_file option or a cache file.
    let mut cache_file_name = String::from("mb-cache-migrate.json");
    let mut cache = MigrateCache::new();

    let solana_opts = parse_solana_config();
    let keypair = Arc::new(parse_keypair(args.keypair, solana_opts));

    let mut mint_accounts = if let Some(candy_machine_id) = args.candy_machine_id {
        println!("Using candy machine id to fetch mint list. . .");
        get_mint_accounts(
            &args.client,
            &Some(candy_machine_id),
            0,
            None,
            false,
            true,
            false,
        )?
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

    // If output file is specified, write to that file.
    if let Some(path) = args.output_file {
        cache_file_name = path;
    }

    let f = if !Path::new(&cache_file_name).exists() {
        File::create(&cache_file_name)?
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(&cache_file_name)?
    };

    let client = Arc::new(args.client);

    let mut counter = 0u8;
    let delay = NANO_SECONDS_IN_SECOND / args.rate_limit;
    let rate_limiter = create_rate_limiter_with_capacity(args.rate_limit as u32, delay as u32);

    // Loop over migrate process so we can retry repeatedly until the user exits.
    loop {
        let remaining_mints = mint_accounts.clone();

        info!("Sending network requests...");
        let pb = create_progress_bar("Sending network requests...", remaining_mints.len() as u64);

        // Create a vector of futures to execute.
        let mut migrate_tasks = Vec::new();

        for mint in remaining_mints {
            let client = client.clone();
            let keypair = keypair.clone();
            let mint_address = args.mint_address.clone();
            let mut rate_limiter = rate_limiter.clone();

            migrate_tasks.push(tokio::spawn({
                rate_limiter.wait();

                let fut = set_and_verify(client, keypair, mint, mint_address, false);
                pb.inc(1);
                fut
            }));
        }
        pb.finish_and_clear();

        let migrate_tasks_len = migrate_tasks.len();

        // Wait for all the tasks to resolve and push the results to our results vector
        let mut migrate_failed = Vec::new();
        let pb = create_progress_bar(
            "Waiting for requests to resolve...",
            migrate_tasks.len() as u64,
        );

        for task in migrate_tasks {
            match task.await.unwrap() {
                Ok(_) => (),
                Err(e) => migrate_failed.push(e),
            }
            pb.inc(1);
        }
        pb.finish_and_clear();

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
            cache.update_errors(migrate_failed);
            cache.write(f)?;
            break;
        }
    }

    Ok(())
}
