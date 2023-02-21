use anyhow::{anyhow, Result as AnyResult};
use async_trait::async_trait;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use log::info;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};
use tokio::sync::Semaphore;

use crate::errors::ActionError;

#[derive(Debug, Deserialize, Serialize)]
pub struct Cache(pub IndexMap<String, CacheItem>);
pub type CacheResults = Vec<Result<(), ActionError>>;

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Cache {
    type Target = IndexMap<String, CacheItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Cache {
    pub fn new() -> Self {
        Cache(IndexMap::new())
    }

    pub fn write<W: Write>(&mut self, writer: W) -> AnyResult<()> {
        self.sort_unstable_keys();
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    pub fn update_errors(&mut self, errors: Vec<Result<(), ActionError>>) {
        let errors = errors.iter().map(|r| r.as_ref()).map(Result::unwrap_err);

        // Clear out old errors.
        self.clear();

        for error in errors {
            match error {
                ActionError::ActionFailed(mint_address, _) => {
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

pub struct BatchActionArgs {
    pub client: RpcClient,
    pub keypair: Keypair,
    pub payer: Option<Keypair>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_value: String,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct RunActionArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub payer: Arc<Option<Keypair>>,
    pub mint_account: String,
    pub new_value: String,
}

#[async_trait]
pub trait Action {
    async fn action(args: RunActionArgs) -> Result<(), ActionError>;

    fn name() -> &'static str;

    async fn run(args: BatchActionArgs) -> AnyResult<()> {
        if args.cache_file.is_some() && args.mint_list.is_some() {
            return Err(anyhow!(
                "Can only specify either a cache or a mint_list file."
            ));
        }

        // Default name, if we don't get an output_file option or a cache file.
        let mut cache_file_name = format!("mb-cache-{}.json", Self::name());
        let mut cache = Cache::new();

        let mut mint_list: Vec<String> = if let Some(mint_list) = args.mint_list {
            let f = File::open(mint_list)?;
            serde_json::from_reader(f)?
        } else if let Some(cache_path) = args.cache_file {
            println!("Retrying items from cache file. . .");
            cache_file_name = cache_path;

            let f = File::open(&cache_file_name)?;
            let cache: Cache = serde_json::from_reader(f)?;
            cache.0.keys().map(|k| k.to_string()).collect()
        } else {
            return Err(anyhow!(
                "Please specify either a n mint_list file or a cache file."
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

        let mut counter = 0u8;
        let client = Arc::new(args.client);
        let keypair = Arc::new(args.keypair);
        let payer = Arc::new(args.payer);

        let semaphore = Arc::new(Semaphore::new(args.batch_size));

        loop {
            let remaining_mints = mint_list.clone();

            info!("Sending network requests...");
            let mut update_tasks = Vec::new();
            println!("Updating NFTs. . .");
            let pb = ProgressBar::new(remaining_mints.len() as u64);

            // Create a vector of futures to execute.
            for mint_address in remaining_mints {
                // Limit total number of concurrent requests to args.batch_size.
                let permit = semaphore.clone().acquire_owned().await.unwrap();

                // Create task to run the action in a separate thread.
                let task = tokio::spawn({
                    let fut = Self::action(RunActionArgs {
                        client: client.clone(),
                        keypair: keypair.clone(),
                        payer: payer.clone(),
                        mint_account: mint_address,
                        new_value: args.new_value.clone(),
                    });

                    // Move the permit into the thread to take ownership of it and then drop it
                    // when the future is complete.
                    drop(permit);

                    fut
                });

                // Collect all the tasks in our futures vector.
                update_tasks.push(task);
            }

            let update_tasks_len = update_tasks.len();

            // Wait for all the tasks to resolve and push the results to our results vector
            let mut update_results = Vec::new();
            for task in update_tasks {
                update_results.push(task.await.unwrap());
                // Increment the counter and update the progress bar.
                pb.inc(1);
            }
            pb.finish_and_clear();

            // Partition migration results.
            let (_update_successful, update_failed): (CacheResults, CacheResults) =
                update_results.into_iter().partition(Result::is_ok);

            // If some of the migrations failed, ask user if they wish to retry and the loop starts again.
            // Otherwise, break out of the loop and write the cache to disk.
            if !update_failed.is_empty() && counter < args.retries {
                counter += 1;
                println!(
                    "{}/{} updates failed. Retrying. . .",
                    &update_failed.len(),
                    update_tasks_len
                );
                cache.update_errors(update_failed);
                mint_list = cache.keys().map(|m| m.to_string()).collect();
            } else if update_failed.is_empty() {
                // None failed so we exit the loop.
                println!("All actions successfully run!");
                break;
            } else {
                println!("Reached max retries. Writing remaining items to cache.");
                cache.write(f)?;
                break;
            }
        }

        Ok(())
    }
}
