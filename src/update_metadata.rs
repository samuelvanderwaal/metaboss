#![allow(unused)]
use anyhow::{anyhow, Result};
use metaplex_token_metadata::{
    instruction::update_metadata_accounts,
    state::{Creator, Data},
};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use std::{fs, str::FromStr};

use crate::constants::*;
use crate::decode::{decode, get_metadata_pda};
use crate::parse::parse_keypair;

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUri {
    mint_account: String,
    new_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUpdateAuthority {
    mint_account: String,
    new_update_authority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTData {
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<JSONCreator>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JSONCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

pub fn update_nft(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    json_file: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let metadata_account = get_metadata_pda(mint_pubkey);

    let f = fs::File::open(json_file)?;
    let new_data: NFTData = serde_json::from_reader(f)?;

    let update_authority = keypair.pubkey();

    // let creators_json = body
    //     .get("properties")
    //     .ok_or_else(|| anyhow!("Bad JSON"))?
    //     .get("creators")
    //     .ok_or_else(|| anyhow!("Bad JSON"))?;

    // let creators = parse_creators(&creators_json)?;

    // let name = parse_name(&body)?;
    // let symbol = parse_symbol(&body)?;
    // let seller_fee_basis_points = parse_seller_fee_basis_points(&body)?;

    let creators = new_data
        .creators
        .ok_or(anyhow!("No creators specified in json file!"))?
        .iter()
        .map(convert_creator)
        .collect::<Result<Vec<Creator>>>()?;

    let data = Data {
        name: new_data.name,
        symbol: new_data.symbol,
        uri: new_data.uri,
        seller_fee_basis_points: new_data.seller_fee_basis_points,
        creators: Some(creators),
    };

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        None,
        Some(data),
        None,
    );
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_new_uri(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    new_uri: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);
    let metadata = decode(client, mint_account)?;

    let mut data = metadata.data;
    data.uri = new_uri.to_string();

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        None,
        Some(data),
        None,
    );

    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_primary_sale_happened(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        None,
        None,
        Some(false),
    );
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_update_authority(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    new_update_authority: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;

    let update_authority = keypair.pubkey();
    let new_update_authority = Pubkey::from_str(new_update_authority)?;

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts(
        program_id,
        metadata_account,
        update_authority,
        Some(new_update_authority),
        None,
        None,
    );
    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&keypair],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}

pub fn set_update_authority_all(
    client: &RpcClient,
    keypair: &String,
    json_file: &String,
) -> Result<()> {
    let file = fs::File::open(json_file)?;
    let items: Vec<NewUpdateAuthority> = serde_json::from_reader(file)?;

    for item in items.iter() {
        println!("Updating metadata for mint account: {}", item.mint_account);
        set_update_authority(
            client,
            keypair,
            &item.mint_account,
            &item.new_update_authority,
        )?;
    }

    Ok(())
}

fn convert_creator(c: &JSONCreator) -> Result<Creator> {
    Ok(Creator {
        address: Pubkey::from_str(&c.address)?,
        verified: c.verified,
        share: c.share,
    })
}

fn parse_creators(creators_json: &Value) -> Result<Vec<Creator>> {
    let mut creators = Vec::new();

    for creator in creators_json
        .as_array()
        .ok_or(anyhow!("Invalid creators array!"))?
    {
        let address = creator
            .get("address")
            .ok_or(anyhow!("Invalid address!"))?
            .as_str()
            .ok_or(anyhow!("Invalid address!"))?
            .parse::<Pubkey>()?;

        let share = creator
            .get("share")
            .ok_or(anyhow!("Invalid share!"))?
            .as_u64()
            .ok_or(anyhow!("Invalid share!"))? as u8;

        creators.push(Creator {
            address,
            verified: false,
            share,
        });
    }

    Ok(creators)
}

fn parse_name(body: &Value) -> Result<String> {
    let name = body
        .get("name")
        .ok_or(anyhow!("Invalid name!"))?
        .as_str()
        .ok_or(anyhow!("Invalid name!"))?
        .to_string();
    Ok(name)
}

fn parse_symbol(body: &Value) -> Result<String> {
    let symbol = body
        .get("symbol")
        .ok_or(anyhow!("Invalid symbol!"))?
        .as_str()
        .ok_or(anyhow!("Invalid symbol!"))?
        .to_string();
    Ok(symbol)
}

fn parse_seller_fee_basis_points(body: &Value) -> Result<u16> {
    let seller_fee_basis_points =
        body.get("seller_fee_basis_points")
            .ok_or(anyhow!("Invalid seller_fee_basis_points!"))?
            .as_u64()
            .ok_or(anyhow!("Invalid seller_fee_basis_points!"))? as u16;
    Ok(seller_fee_basis_points)
}
