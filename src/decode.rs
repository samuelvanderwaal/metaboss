use anyhow::{anyhow, Result as AnyResult};
use borsh::BorshDeserialize;
use indicatif::ParallelProgressIterator;
use log::{debug, error, info};
use metaboss_lib::decode::{
    decode_edition_from_mint, decode_edition_marker_from_mint, decode_master_edition_from_mint,
};
use mpl_token_metadata::state::CollectionDetails;
use mpl_token_metadata::state::{Key, Metadata, TokenStandard, UseMethod};
use rayon::prelude::*;
use retry::{delay::Exponential, retry};
use serde::Serialize;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::fs::File;
use std::str::FromStr;

use crate::constants::*;
use crate::errors::*;
use crate::limiter::create_default_rate_limiter;
use crate::parse::is_only_one_option;

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
            let metadata = match decode(client, mint_account) {
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

            debug!(
                "Converting metadata into JSON for mint account {}",
                mint_account
            );
            let json_metadata = match decode_to_json(metadata, full) {
                Ok(j) => j,
                Err(err) => {
                    error!(
                        "Failed to decode metadata to JSON for mint account: {}, error: {}",
                        mint_account, err
                    );
                    return;
                }
            };

            debug!("Creating file for mint account: {}", mint_account);
            let mut file = match File::create(format!("{}/{}.json", output, mint_account)) {
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
            match serde_json::to_writer_pretty(&mut file, &json_metadata) {
                Ok(_) => (),
                Err(err) => {
                    error!(
                        "Failed to write JSON file for mint account: {}, error: {}",
                        mint_account, err
                    );
                }
            }
        });

    Ok(())
}

pub fn decode_master_edition(client: &RpcClient, mint_account: &str) -> AnyResult<()> {
    let master_edition = decode_master_edition_from_mint(client, mint_account)?;
    println!("{:?}", master_edition);

    Ok(())
}

pub fn decode_print_edition(client: &RpcClient, mint_account: &str) -> AnyResult<()> {
    let print_edition = decode_edition_from_mint(client, mint_account)?;
    println!("{:?}", print_edition);

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
    println!("{:?}", edition_marker);

    Ok(())
}

pub fn decode_metadata(
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
            println!("{:?}", data);
            return Ok(());
        }
        let metadata = decode(client, mint_account)?;
        let json_metadata = decode_to_json(metadata, full)?;
        let mut file = File::create(format!("{}/{}.json", output, mint_account))?;
        serde_json::to_writer_pretty(&mut file, &json_metadata)?;
    } else if let Some(list_path) = list_path {
        decode_metadata_all(client, list_path, full, output)?;
    } else {
        return Err(anyhow!(
            "Please specify either a mint account or a list of mint accounts, but not both."
        ));
    };

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

    let metadata: Metadata = match Metadata::deserialize(&mut account_data.as_slice()) {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(metadata)
}

fn decode_to_json(metadata: Metadata, full: bool) -> AnyResult<Value> {
    let mut creators: Vec<JSONCreator> = Vec::new();

    if let Some(c) = metadata.data.creators {
        creators = c
            .iter()
            .map(|c| JSONCreator {
                address: c.address.to_string(),
                verified: c.verified,
                share: c.share,
            })
            .collect::<Vec<JSONCreator>>();
    }

    let data_json = json!({
        "name": metadata.data.name.trim_matches(char::from(0)),
        "symbol": metadata.data.symbol.trim_matches(char::from(0)),
        "seller_fee_basis_points": metadata.data.seller_fee_basis_points,
        "uri": metadata.data.uri.trim_matches(char::from(0)),
        "creators": creators,
    });

    if !full {
        return Ok(data_json);
    }

    let mut token_standard: Option<String> = None;
    if let Some(ts) = metadata.token_standard {
        token_standard = Some(parse_token_standard(ts))
    }

    let mut collection: Option<JSONCollection> = None;
    if let Some(c) = metadata.collection {
        collection = Some(JSONCollection {
            verified: c.verified,
            key: c.key.to_string(),
        })
    }

    let mut collection_details: Option<JSONCollectionDetails> = None;
    if let Some(details) = metadata.collection_details {
        match details {
            CollectionDetails::V1 { size } => {
                collection_details = Some(JSONCollectionDetails::V1 { size })
            }
        }
    }

    let mut uses: Option<JSONUses> = None;
    if let Some(u) = metadata.uses {
        uses = Some(JSONUses {
            use_method: parse_use_method(u.use_method),
            remaining: u.remaining,
            total: u.total,
        })
    }

    let json_metadata = json!({
        "key": parse_key(metadata.key),
        "update_authority": metadata.update_authority.to_string(),
        "mint_account": metadata.mint.to_string(),
        "nft_data": data_json,
        "primary_sale_happened": metadata.primary_sale_happened,
        "is_mutable": metadata.is_mutable,
        "edition_nonce": metadata.edition_nonce,
        "token_standard": token_standard,
        "collection": collection,
        "uses": uses,
        "collection_details": collection_details,
    });
    Ok(json_metadata)
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

fn parse_key(key: Key) -> String {
    match key {
        Key::Uninitialized => String::from("Uninitialized"),
        Key::EditionV1 => String::from("EditionV1"),
        Key::MasterEditionV1 => String::from("MasterEditionV1"),
        Key::ReservationListV1 => String::from("ReservationListV1"),
        Key::MetadataV1 => String::from("MetadataV1"),
        Key::ReservationListV2 => String::from("ReservationListV2"),
        Key::MasterEditionV2 => String::from("MasterEditionV2"),
        Key::EditionMarker => String::from("EditionMarker"),
        Key::UseAuthorityRecord => String::from("UseAuthorityRecord"),
        Key::CollectionAuthorityRecord => String::from("CollectionAuthorityRecord"),
    }
}

fn parse_token_standard(token_standard: TokenStandard) -> String {
    match token_standard {
        TokenStandard::NonFungible => String::from("NonFungible"),
        TokenStandard::FungibleAsset => String::from("FungibleAsset"),
        TokenStandard::Fungible => String::from("Fungible"),
        TokenStandard::NonFungibleEdition => String::from("NonFungibleEdition"),
    }
}

fn parse_use_method(use_method: UseMethod) -> String {
    match use_method {
        UseMethod::Burn => String::from("Burn"),
        UseMethod::Single => String::from("Single"),
        UseMethod::Multiple => String::from("Multiple"),
    }
}
