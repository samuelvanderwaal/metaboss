use anyhow::{anyhow, Result as AnyResult};
use metaplex_token_metadata::state::{Key, Metadata};
use serde::Serialize;
use serde_json::{json, Value};
use solana_client::rpc_client::RpcClient;
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::pubkey::Pubkey;
use std::fs;
use std::str::FromStr;

use crate::constants::*;
use crate::errors::*;

#[derive(Debug, Serialize)]
pub struct JSONCreator {
    pub address: String,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

pub fn decode_metadata_all(
    client: &RpcClient,
    json_file: &String,
    output: &String,
) -> AnyResult<()> {
    let file = fs::File::open(json_file)?;
    let mint_accounts: Vec<String> = serde_json::from_reader(file)?;

    for mint_account in &mint_accounts {
        let metadata = match decode(client, mint_account) {
            Ok(m) => m,
            Err(err) => match err {
                DecodeError::MissingAccount(account) => {
                    println!("No account data found for mint account: {}!", account);
                    continue;
                }
                _ => return Err(anyhow!(err)),
            },
        };

        let json_metadata = decode_to_json(metadata)?;

        let mut file = fs::File::create(format!("{}/{}.json", output, mint_account))?;
        serde_json::to_writer(&mut file, &json_metadata)?;
    }

    Ok(())
}

pub fn decode_metadata(
    client: &RpcClient,
    mint_account: &String,
    output: &String,
) -> AnyResult<()> {
    let metadata = decode(client, mint_account)?;
    let json_metadata = decode_to_json(metadata)?;

    let mut file = fs::File::create(format!("{}/{}.json", output, mint_account))?;
    serde_json::to_writer(&mut file, &json_metadata)?;
    Ok(())
}

pub fn decode(client: &RpcClient, mint_account: &String) -> Result<Metadata, DecodeError> {
    let pubkey = match Pubkey::from_str(&mint_account) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed),
    };
    let metadata_pda = get_metadata_pda(pubkey);

    let account_data = match client.get_account_data(&metadata_pda) {
        Ok(data) => data,
        Err(_) => {
            return Err(DecodeError::MissingAccount(mint_account.to_string()));
        }
    };

    let metadata: Metadata = match try_from_slice_unchecked(&account_data) {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(metadata)
}

fn decode_to_json(metadata: Metadata) -> AnyResult<Value> {
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
        "name": metadata.data.name.to_string().trim_matches(char::from(0)),
        "symbol": metadata.data.symbol.to_string().trim_matches(char::from(0)),
        "seller_fee_basis_points": metadata.data.seller_fee_basis_points,
        "uri": metadata.data.uri.to_string().trim_matches(char::from(0)),
        "creators": [creators],
    });

    let json_metadata = json!({
        "key": parse_key(metadata.key),
        "update_authority": metadata.update_authority.to_string(),
        "mint": metadata.mint.to_string(),
        "data": data_json,
        "primary_sale_happened": metadata.primary_sale_happened,
        "is_mutable": metadata.is_mutable,
        "edition_nonce": metadata.edition_nonce,
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
    }
}
