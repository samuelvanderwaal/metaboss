use std::{fs::File, path::PathBuf};

use anyhow::Result;
use reqwest::header::HeaderMap;
use serde_json::json;
use solana_program::pubkey::Pubkey;

use crate::{
    helius::ByCreatorResponse,
    setup::{CliConfig, ClientLike, ClientType},
    spinner::create_spinner,
};

pub struct FcvaArgs {
    pub rpc_url: String,
    pub creator: Pubkey,
    pub output: PathBuf,
}

pub async fn fcva_mints(args: FcvaArgs) -> Result<()> {
    let config = CliConfig::new(None, Some(args.rpc_url), ClientType::DAS)?;

    let creator = args.creator.to_string();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let client = match config.client {
        ClientLike::DasClient(client) => client,
        _ => panic!("Wrong client type"),
    };

    let mut mints = Vec::new();
    let mut page = 1;
    let spinner = create_spinner("Getting assets...");
    loop {
        let body = json!(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAssetsByCreator",
            "params": {
                "creatorAddress": args.creator.to_string(),
                "onlyVerified": true,
                "page": page,
                "limit": 1000,
                "displayOptions":
                {
                    "showUnverifiedCollections": false,
                    "showCollectionMetadata": false,
                    "showInscription": false,
                }
            },
        });

        let response = client
            .post(config.rpc_url.clone())
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;

        let res: ByCreatorResponse = response.json().await?;

        if res.result.items.is_empty() {
            break;
        }

        page += 1;

        res.result
            .items
            .iter()
            .filter(|item| {
                item.creators.first().is_some()
                    && item.creators.first().unwrap().address.to_string() == creator
            })
            .for_each(|item| {
                mints.push(item.id.clone());
            });
    }
    spinner.finish();

    mints.sort();

    // Write to file
    let file = File::create(format!("{}_mints.json", args.creator))?;
    serde_json::to_writer_pretty(file, &mints)?;

    // let mut mints: Vec<String> = assets.iter().map(|asset| asset.mint.clone()).collect();

    // Ok(mints)

    Ok(())
}
