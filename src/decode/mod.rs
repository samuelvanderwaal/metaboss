use anyhow::{anyhow, Result as AnyResult};
use borsh::BorshDeserialize;
use indicatif::ParallelProgressIterator;
use log::{debug, error, info};
use metaboss_lib::data::NftData;
use metaboss_lib::decode::{
    decode_bpf_loader_upgradeable_state, decode_edition_from_mint, decode_edition_marker_from_mint,
    decode_master_edition_from_mint, decode_mint, decode_token,
};
use mpl_token_metadata::accounts::Metadata;
use rayon::prelude::*;
use retry::{delay::Exponential, retry};
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::fs::File;
use std::str::FromStr;

use crate::constants::*;
use crate::errors::*;
use crate::limiter::create_default_rate_limiter;
use crate::parse::is_only_one_option;

mod rule_set;
pub use rule_set::*;

#[derive(Debug, Serialize)]
pub struct JSONCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

#[derive(Debug, Serialize)]
pub struct JSONCollection {
    pub verified: bool,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub enum JSONCollectionDetails {
    V1 { size: u64 },
}

#[derive(Debug, Serialize)]
pub struct JSONUses {
    pub use_method: String,
    pub remaining: u64,
    pub total: u64,
}

pub fn decode_metadata_all(
    client: &RpcClient,
    json_file: &str,
    full: bool,
    output: &str,
) -> AnyResult<()> {
    let file = File::open(json_file)?;
    let mint_accounts: Vec<String> = serde_json::from_reader(file)?;
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    info!("Decoding accounts...");
    println!("Decoding accounts...");
    mint_accounts
        .par_iter()
        .progress()
        .for_each(|mint_account| {
            let mut handle = handle.clone();
            if use_rate_limit {
                handle.wait();
            }

            debug!("Decoding metadata for mint account: {}", mint_account);
            let mut metadata = match decode(client, mint_account) {
                Ok(m) => m,
                Err(err) => match err {
                    DecodeError::ClientError(kind) => {
                        error!("Client Error: {}!", kind);
                        return;
                    }
                    DecodeError::PubkeyParseFailed(address) => {
                        error!("Failed to parse pubkey from mint address: {}", address);
                        return;
                    }
                    err => {
                        error!(
                            "Failed to decode metadata for mint account: {}, error: {}",
                            mint_account, err
                        );
                        return;
                    }
                },
            };
            metadata.name = metadata.name.replace('\u{0}', "");
            metadata.uri = metadata.uri.replace('\u{0}', "");
            metadata.symbol = metadata.symbol.replace('\u{0}', "");

            debug!("Creating file for mint account: {}", mint_account);
            let mut file = match File::create(format!("{output}/{mint_account}.json")) {
                Ok(f) => f,
                Err(err) => {
                    error!(
                        "Failed to create JSON file for mint account: {}, error: {}",
                        mint_account, err
                    );
                    return;
                }
            };

            debug!("Writing to file for mint account: {}", mint_account);
            if full {
                match serde_json::to_writer_pretty(&mut file, &metadata) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Failed to write to JSON file for mint account: {}, error: {}",
                            mint_account, err
                        );
                    }
                }
            } else {
                let data = NftData::from(metadata);

                match serde_json::to_writer_pretty(&mut file, &data) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Failed to write to JSON file for mint account: {}, error: {}",
                            mint_account, err
                        );
                    }
                }
            }
        });

    Ok(())
}

pub fn decode_master_edition(client: &RpcClient, mint_account: &str) -> AnyResult<()> {
    let master_edition = decode_master_edition_from_mint(client, mint_account)?;
    println!("{master_edition:?}");

    Ok(())
}

pub fn decode_print_edition(client: &RpcClient, mint_account: &str) -> AnyResult<()> {
    let print_edition = decode_edition_from_mint(client, mint_account)?;
    println!("{print_edition:?}");

    Ok(())
}

pub fn decode_edition_marker(
    client: &RpcClient,
    mint_account: &str,
    edition_num: Option<u64>,
    marker_num: Option<u64>,
) -> AnyResult<()> {
    let edition_num = if let Some(num) = edition_num {
        num
    } else if let Some(num) = marker_num {
        num * 248
    } else {
        return Err(anyhow!("Edition or marker number is required"));
    };

    let edition_marker = decode_edition_marker_from_mint(client, mint_account, edition_num)?;
    println!("{edition_marker:?}");

    Ok(())
}

pub fn decode_metadata_from_mint(
    client: &RpcClient,
    account: Option<&String>,
    full: bool,
    list_path: Option<&String>,
    raw: bool,
    output: &str,
) -> AnyResult<()> {
    // Explicitly warn the user if they provide incorrect options combinations
    if !is_only_one_option(&account, &list_path) {
        return Err(anyhow!(
            "Please specify either a mint account or a list of mint accounts, but not both."
        ));
    }

    if let Some(mint_account) = account {
        if raw {
            let data = decode_raw(client, mint_account)?;
            println!("{data:?}");
            return Ok(());
        }
        let mut metadata = decode(client, mint_account)?;
        metadata.name = metadata.name.replace('\u{0}', "");
        metadata.uri = metadata.uri.replace('\u{0}', "");
        metadata.symbol = metadata.symbol.replace('\u{0}', "");

        let mut file = File::create(format!("{output}/{mint_account}.json"))?;

        if full {
            serde_json::to_writer_pretty(&mut file, &metadata)?;
        } else {
            let data = NftData::from(metadata);
            serde_json::to_writer_pretty(&mut file, &data)?;
        }
    } else if let Some(list_path) = list_path {
        decode_metadata_all(client, list_path, full, output)?;
    } else {
        return Err(anyhow!(
            "Please specify either a mint account or a list of mint accounts, but not both."
        ));
    };

    Ok(())
}

pub fn decode_metadata(client: &RpcClient, account: String, output: &str) -> AnyResult<()> {
    let pubkey = Pubkey::from_str(&account)?;
    let metadata = metaboss_lib::decode::decode_metadata(client, &pubkey)?;
    let mut file = File::create(format!("{output}/{account}.json"))?;
    serde_json::to_writer_pretty(&mut file, &metadata)?;

    Ok(())
}

pub fn decode_mint_account(client: &RpcClient, mint_account: &str) -> AnyResult<()> {
    let mint = decode_mint(client, mint_account)?;
    println!("{mint:?}");

    Ok(())
}

pub fn decode_token_account(client: &RpcClient, token_account: &str) -> AnyResult<()> {
    let account = decode_token(client, token_account)?;
    println!("{account:?}");

    Ok(())
}

pub fn decode_token_record_from_mint(client: &RpcClient, mint: &str) -> AnyResult<()> {
    let pubkey = Pubkey::from_str(mint)?;
    let token_record = metaboss_lib::decode::decode_token_record_from_mint(client, pubkey)?;
    println!("{token_record:?}");

    Ok(())
}

pub fn decode_raw(client: &RpcClient, mint_account: &str) -> Result<Vec<u8>, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_account) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_account.to_string())),
    };
    let metadata_pda = get_metadata_pda(pubkey);

    let account_data = match retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.get_account_data(&metadata_pda),
    ) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::NetworkError(err.to_string()));
        }
    };
    Ok(account_data)
}

pub fn decode(client: &RpcClient, mint_account: &str) -> Result<Metadata, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_account) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_account.to_string())),
    };
    let metadata_pda = get_metadata_pda(pubkey);

    let account_data = match retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.get_account_data(&metadata_pda),
    ) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::NetworkError(err.to_string()));
        }
    };

    Metadata::deserialize(&mut account_data.as_slice())
        .map_err(|e| DecodeError::DecodeMetadataFailed(e.to_string()))
}

pub fn process_decode_bpf_loader_upgradable_state(
    client: &RpcClient,
    address: &str,
) -> AnyResult<()> {
    let state = decode_bpf_loader_upgradeable_state(client, address)?;

    println!("{state:?}");

    Ok(())
}

pub fn get_metadata_pda(pubkey: Pubkey) -> Pubkey {
    let metaplex_pubkey = METAPLEX_PROGRAM_ID
        .parse::<Pubkey>()
        .expect("Failed to parse Metaplex Program Id");

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}
