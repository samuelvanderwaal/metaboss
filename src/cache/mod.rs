use anyhow::{anyhow, Result as AnyResult};
use async_trait::async_trait;
use indexmap::IndexMap;
use log::info;
use metaboss_lib::data::Priority;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Write,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
};

use crate::{
    constants::NANO_SECONDS_IN_SECOND, errors::ActionError,
    limiter::create_rate_limiter_with_capacity, spinner::create_progress_bar, utils::find_tm_error,
};

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

        //Regex to find hex codes in error
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r" 0x[0-9a-fA-F]+").expect("Failed to create regex"));

        for error in errors {
            match error {
                ActionError::ActionFailed(mint_address, _) => {
                    // Find hex codes in error message.
                    let error_message = if let Some(mat) = RE.find(&error.to_string()) {
                        find_tm_error(&mat.as_str().trim_start().replace("0x", ""))
                            .unwrap_or_else(|| error.to_string())
                    } else {
                        error.to_string()
                    };

                    let item = CacheItem {
                        error: Some(error_message),
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

pub type MintValues = HashMap<String, String>;

pub enum NewValue {
    None,
    Single(String),
    List(MintValues),
}

pub struct BatchActionArgs {
    pub client: RpcClient,
    pub keypair: Keypair,
    pub payer: Option<Keypair>,
    pub mint_list: Option<Vec<String>>,
    pub cache_file: Option<String>,
    pub new_value: NewValue,
    pub should_append: bool,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct RunActionArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub payer: Arc<Option<Keypair>>,
    pub mint_account: String,
    pub new_value: String,
    pub should_append: bool,
    pub priority: Priority,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ActionError;

    #[test]
    fn test_cache_new_is_empty() {
        // Act
        let cache = Cache::new();

        // Assert
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_default_is_empty() {
        // Act
        let cache = Cache::default();

        // Assert
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_write_produces_valid_json() {
        // Arrange
        let mut cache = Cache::new();
        cache.insert(
            "mint123".to_string(),
            CacheItem {
                error: Some("test error".to_string()),
            },
        );

        // Act
        let mut buf = Vec::new();
        cache.write(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();

        // Assert
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("mint123").is_some());
    }

    #[test]
    fn test_cache_update_errors_extracts_mint_addresses() {
        // Arrange
        let mut cache = Cache::new();
        let errors: Vec<Result<(), ActionError>> = vec![
            Err(ActionError::ActionFailed(
                "mintAAA".to_string(),
                "some error".to_string(),
            )),
            Err(ActionError::ActionFailed(
                "mintBBB".to_string(),
                "another error".to_string(),
            )),
        ];

        // Act
        cache.update_errors(errors);

        // Assert
        assert_eq!(cache.len(), 2);
        assert!(cache.contains_key("mintAAA"));
        assert!(cache.contains_key("mintBBB"));
    }

    #[test]
    fn test_cache_update_errors_clears_old_errors() {
        // Arrange
        let mut cache = Cache::new();
        cache.insert(
            "old_mint".to_string(),
            CacheItem {
                error: Some("old error".to_string()),
            },
        );

        let errors: Vec<Result<(), ActionError>> = vec![Err(ActionError::ActionFailed(
            "new_mint".to_string(),
            "new error".to_string(),
        ))];

        // Act
        cache.update_errors(errors);

        // Assert
        assert_eq!(cache.len(), 1);
        assert!(!cache.contains_key("old_mint"));
        assert!(cache.contains_key("new_mint"));
    }

    #[test]
    fn test_cache_update_errors_hex_code_in_message() {
        // Arrange
        let mut cache = Cache::new();
        let errors: Vec<Result<(), ActionError>> = vec![Err(ActionError::ActionFailed(
            "mintHEX".to_string(),
            "Transaction failed with 0x1771".to_string(),
        ))];

        // Act
        cache.update_errors(errors);

        // Assert
        assert_eq!(cache.len(), 1);
        let item = cache.get("mintHEX").unwrap();
        // The error message should have been processed (hex code extracted).
        // Whether it resolves to a known error or falls back to the original,
        // it should have a non-empty error string.
        assert!(item.error.is_some());
        assert!(!item.error.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_cache_update_errors_no_hex_code() {
        // Arrange
        let mut cache = Cache::new();
        let errors: Vec<Result<(), ActionError>> = vec![Err(ActionError::ActionFailed(
            "mintNOHEX".to_string(),
            "Simple error without hex".to_string(),
        ))];

        // Act
        cache.update_errors(errors);

        // Assert
        let item = cache.get("mintNOHEX").unwrap();
        // Without a hex code, the error message should be the full ActionError display string.
        assert!(item
            .error
            .as_ref()
            .unwrap()
            .contains("Simple error without hex"));
    }
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
            mint_list
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

        let delay = NANO_SECONDS_IN_SECOND / args.rate_limit;
        let rate_limiter = create_rate_limiter_with_capacity(args.rate_limit as u32, delay as u32);

        loop {
            let remaining_mints = mint_list.clone();

            let mint_length = remaining_mints.len();

            info!("Sending network requests...");
            let mut update_tasks = Vec::new();
            let pb =
                create_progress_bar("Sending network requests...", remaining_mints.len() as u64);

            // Create a vector of futures to execute.
            for mint_address in remaining_mints {
                let mut rate_limiter = rate_limiter.clone();

                let empty_string = String::new();

                let new_value = match &args.new_value {
                    NewValue::None => &empty_string,
                    NewValue::Single(value) => value,
                    NewValue::List(values) => match values.get(&mint_address) {
                        Some(v) => v,
                        None => {
                            return Err(ActionError::ActionFailed(
                                mint_address.clone(),
                                "mint found in cache but missing from input list".to_string(),
                            )
                            .into());
                        }
                    },
                };

                // Create task to run the action in a separate thread.
                let task = tokio::spawn({
                    rate_limiter.wait();
                    let fut = Self::action(RunActionArgs {
                        client: client.clone(),
                        keypair: keypair.clone(),
                        payer: payer.clone(),
                        mint_account: mint_address,
                        new_value: new_value.to_string(),
                        should_append: args.should_append,
                        priority: args.priority.clone(),
                    });

                    pb.inc(1);

                    fut
                });

                // Collect all the tasks in our futures vector.
                update_tasks.push(task);
            }

            pb.finish_and_clear();

            let update_tasks_len = update_tasks.len();

            let pb = create_progress_bar("Waiting for requests to resolve...", mint_length as u64);

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

            println!("Updates failed: {}", update_failed.len());

            // If some of the migrations failed, check the retry count and re-run if appropriate,
            // otherwise, break out of the loop and write the cache to disk.
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
                cache.update_errors(update_failed);
                cache.write(f)?;
                break;
            }
        }

        Ok(())
    }
}
