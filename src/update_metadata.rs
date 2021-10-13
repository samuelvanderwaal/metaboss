use anyhow::{anyhow, Result};
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use spl_token_metadata::{
    instruction::update_metadata_accounts,
    state::{Creator, Data},
};
use std::{fs, str::FromStr, thread, time::Duration};

use crate::constants::*;
use crate::decode::get_metadata_pda;
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

pub fn update_nft(
    client: &RpcClient,
    keypair: &String,
    mint_account: &String,
    new_uri: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let metadata_account = get_metadata_pda(mint_pubkey);

    let update_authority = keypair.pubkey();

    let body: Value = reqwest::blocking::get(new_uri)?.json()?;

    let creators_json = body
        .get("properties")
        .ok_or_else(|| anyhow!("Bad JSON"))?
        .get("creators")
        .ok_or_else(|| anyhow!("Bad JSON"))?;

    let creators = parse_creators(&creators_json)?;

    let name = parse_name(&body)?;
    let symbol = parse_symbol(&body)?;
    let seller_fee_basis_points = parse_seller_fee_basis_points(&body)?;

    let data = Data {
        name,
        symbol,
        uri: new_uri.to_string(),
        seller_fee_basis_points,
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

pub fn update_nft_all(client: &RpcClient, keypair: &String, json_file: &String) -> Result<()> {
    let file = fs::File::open(json_file)?;
    let items: Vec<NewUri> = serde_json::from_reader(file)?;

    for item in items.iter() {
        println!("Updating metadata for mint account: {}", item.mint_account);
        update_nft(client, keypair, &item.mint_account, &item.new_uri)?;
        thread::sleep(Duration::from_millis(DEFAULT_RPC_DELAY_MS));
    }

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
        update_nft(
            client,
            keypair,
            &item.mint_account,
            &item.new_update_authority,
        )?;
    }

    Ok(())
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
