use anyhow::{anyhow, Result};
use log::{info, warn};
use mpl_token_metadata::state::DataV2;
use solana_client::rpc_client::RpcClient;
use std::cmp;
use std::fs::File;

use crate::decode::decode;
use crate::errors::UpdateError;
use crate::parse::parse_solana_config;
use crate::parse::{parse_cli_creators, parse_keypair};
use crate::spinner::create_alt_spinner;

use super::update_data;

pub type UpdateResults = Vec<Result<(), UpdateError>>;

pub async fn update_creator_by_position(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_creators: &str,
    should_append: bool,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_creators = old_md.data;
    let parsed_creators = parse_cli_creators(new_creators.to_string(), should_append)?;

    let new_creators = if let Some(mut old_creators) = data_with_old_creators.creators {
        if !should_append {
            parsed_creators
        } else {
            let remaining_space = 5 - old_creators.len();
            warn!(
                "Appending {} new creators with old creators with shares of 0",
                parsed_creators.len()
            );
            let end_index = cmp::min(parsed_creators.len(), remaining_space);
            old_creators.append(&mut parsed_creators[0..end_index].to_vec());
            old_creators
        }
    } else {
        parsed_creators
    };

    let shares = new_creators.iter().fold(0, |acc, c| acc + c.share);
    if shares != 100 {
        return Err(anyhow!("Creators shares must sum to 100!"));
    }

    let new_data = DataV2 {
        creators: Some(new_creators),
        seller_fee_basis_points: data_with_old_creators.seller_fee_basis_points,
        name: data_with_old_creators.name,
        symbol: data_with_old_creators.symbol,
        uri: data_with_old_creators.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &keypair, mint_account, new_data)?;
    Ok(())
}

pub async fn update_creator_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_list: &str,
    new_creators: &str,
    should_append: bool,
    retries: u8,
) -> Result<()> {
    let f = File::open(mint_list)?;
    let mints: Vec<String> = serde_json::from_reader(f)?;

    let mut counter = 0u8;

    loop {
        let remaining_mints = mints.clone();

        info!("Sending network requests...");
        let spinner = create_alt_spinner("Sending network requests....");
        // Create a vector of futures to execute.
        let update_tasks: Vec<_> = remaining_mints
            .into_iter()
            .map(|mint_address| {
                tokio::spawn(update_creator_by_position(
                    client.clone(),
                    keypair_path.clone(),
                    &mint_address,
                    new_creators,
                    false,
                ))
            })
            .collect();
        spinner.finish();

        let update_tasks_len = update_tasks.len();

        // Wait for all the tasks to resolve and push the results to our results vector
        let mut update_results = Vec::new();
        let spinner = create_alt_spinner("Awaiting results...");
        for task in update_tasks {
            update_results.push(task.await.unwrap());
        }
        spinner.finish();

        // Partition migration results.
        let (_update_successful, update_failed): (UpdateResults, UpdateResults) =
            update_results.into_iter().partition(Result::is_ok);

        // If some of the migrations failed, ask user if they wish to retry and the loop starts again.
        // Otherwise, break out of the loop and write the cache to disk.
        if !update_failed.is_empty() && counter < retries {
            counter += 1;
            println!(
                "{}/{} migrations failed. Retrying. . .",
                &update_failed.len(),
                update_tasks_len
            );
            cache.update_errors(update_failed);
            mints = cache.0.keys().map(|m| m.to_string()).collect();
        } else if update_failed.is_empty() {
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
