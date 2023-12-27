use std::{fmt::Display, fs::File, path::PathBuf, str::FromStr};

use anyhow::Result;
use metaboss_lib::derive::derive_metadata_pda;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;

use crate::{
    setup::{CliConfig, ClientLike, ClientType},
    spinner::create_spinner,
};

use super::{DasResponse, Holder, Item};

#[derive(Debug)]
pub enum GroupKey {
    Mint,
    Fvca,
    Mcc,
}

impl FromStr for GroupKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mint" => Ok(GroupKey::Mint),
            "fvca" => Ok(GroupKey::Fvca),
            "mcc" => Ok(GroupKey::Mcc),
            _ => Err(format!("Invalid group key: {}", s)),
        }
    }
}

impl Display for GroupKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupKey::Mint => write!(f, "mint"),
            GroupKey::Fvca => write!(f, "fvca"),
            GroupKey::Mcc => write!(f, "mcc"),
        }
    }
}

pub struct HoldersArgs {
    pub rpc_url: String,
    pub group_key: GroupKey,
    pub group_value: Pubkey,
    pub output: PathBuf,
}

struct Query {
    method: String,
    params: Value,
    fvca_filter: bool,
}

pub async fn snapshot_holders(args: HoldersArgs) -> Result<()> {
    let config = CliConfig::new(None, Some(args.rpc_url), ClientType::DAS)?;

    let query = match args.group_key {
        GroupKey::Mint => todo!(),
        GroupKey::Fvca => Query {
            method: "getAssetsByCreator".to_string(),
            params: json!({
                "creatorAddress": args.group_value.to_string(),
                "onlyVerified": true,
                "page": 1,
                "limit": 1000
            }),
            fvca_filter: true,
        },
        GroupKey::Mcc => Query {
            method: "getAssetsByGroup".to_string(),
            params: json!({
                "groupKey": "collection",
                "groupValue": args.group_value.to_string(),
                "page": 1,
                "limit": 1000
            }),
            fvca_filter: false,
        },
    };

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let client = match config.client {
        ClientLike::DasClient(client) => client,
        _ => panic!("Wrong client type"),
    };

    let mut holders = Vec::new();
    let mut page = 1;

    let mut body = json!(
    {
        "jsonrpc": "2.0",
        "id": 1,
        "method": query.method,
        "params": query.params,
    });

    let fvca_filter = |item: &Item| {
        item.creators.first().is_some()
            && item.creators.first().unwrap().address.to_string() == args.group_value.to_string()
    };

    let spinner = create_spinner("Getting assets...");
    loop {
        let response = client
            .post(config.rpc_url.clone())
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;

        let res: DasResponse = response.json().await?;

        if res.result.items.is_empty() {
            break;
        }

        page += 1;
        body["params"]["page"] = json!(page);

        res.result
            .items
            .iter()
            .filter(|item| {
                if query.fvca_filter {
                    fvca_filter(item)
                } else {
                    true
                }
            })
            .for_each(|item| {
                let mint_address = item.id.clone();
                let metadata_pubkey =
                    derive_metadata_pda(&Pubkey::from_str(mint_address.as_str()).unwrap());
                let owner_address = item.ownership.owner.clone();
                let ata_pubkey = get_associated_token_address(
                    &Pubkey::from_str(&owner_address).unwrap(),
                    &Pubkey::from_str(&mint_address).unwrap(),
                );

                holders.push(Holder {
                    owner_wallet: owner_address,
                    mint_account: item.id.clone(),
                    metadata_account: metadata_pubkey.to_string(),
                    associated_token_address: ata_pubkey.to_string(),
                });
            });
    }
    spinner.finish();

    holders.sort();

    // Write to file
    let file = File::create(format!(
        "{}_{}_holders.json",
        args.group_value, args.group_key
    ))?;
    serde_json::to_writer_pretty(file, &holders)?;

    Ok(())
}

pub struct FcvaArgs {
    pub rpc_url: String,
    pub creator: Option<Pubkey>,
    pub output: PathBuf,
}

pub async fn fcva_mints(args: FcvaArgs) -> Result<()> {
    let config = CliConfig::new(None, Some(args.rpc_url), ClientType::DAS)?;

    // Prioritize creator from args, then config, then fail.
    let creator = if let Some(creator) = args.creator {
        creator.to_string()
    } else if let Some(creator) = config.keypair {
        creator.pubkey().to_string()
    } else {
        panic!("No creator provided");
    };

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
                "creatorAddress": creator,
                "onlyVerified": true,
                "page": page,
                "limit": 1000
            },
        });

        let response = client
            .post(config.rpc_url.clone())
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;

        let res: DasResponse = response.json().await?;

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
    let file = File::create(format!("{}_mints.json", creator))?;
    serde_json::to_writer_pretty(file, &mints)?;

    Ok(())
}

pub struct MccArgs {
    pub rpc_url: String,
    pub mcc_id: Pubkey,
    pub output: PathBuf,
}

pub async fn mcc_mints(args: MccArgs) -> Result<()> {
    let config = CliConfig::new(None, Some(args.rpc_url), ClientType::DAS)?;

    let mcc_id = args.mcc_id.to_string();

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    let client = match config.client {
        ClientLike::DasClient(client) => client,
        _ => panic!("Wrong client type"),
    };

    let mut mints: Vec<String> = Vec::new();
    let mut page = 1;
    let spinner = create_spinner("Getting assets...");
    loop {
        let body = json!(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAssetsByGroup",
            "params": {
                "groupKey": "collection",
                "groupValue": mcc_id,
                "page": page,
                "limit": 1000
            },
        });

        let response = client
            .post(config.rpc_url.clone())
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;

        let res: DasResponse = response.json().await?;

        if res.result.items.is_empty() {
            break;
        }

        page += 1;

        res.result.items.iter().for_each(|item| {
            mints.push(item.id.clone());
        });
    }
    spinner.finish_and_clear();

    mints.sort();

    // Write to file
    let file = File::create(format!("{}_mints.json", mcc_id))?;
    serde_json::to_writer_pretty(file, &mints)?;

    Ok(())
}
