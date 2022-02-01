use anyhow::{anyhow, Context, Result};
use mpl_token_metadata::state::{Creator, Data};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::Keypair;
use std::{env, fs, path::Path, str::FromStr};

use crate::data::{NFTCreator, NFTData};

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

pub fn first_creator_is_verified(creators_opt: &Option<Vec<Creator>>) -> bool {
    // Only add mints with a verified creator.
    if let Some(creators) = creators_opt {
        if creators[0].verified {
            return true;
        }
    }
    false
}

pub fn parse_keypair(path: &String) -> Result<Keypair> {
    let secret_string = fs::read_to_string(path).context("Can't find key file")?;

    // Try to decode the secret string as a JSON array of ints first and then as a base58 encoded string to support Phantom private keys.
    let secret_bytes: Vec<u8> = match serde_json::from_str(&secret_string) {
        Ok(bytes) => bytes,
        Err(_) => match bs58::decode(&secret_string.trim()).into_vec() {
            Ok(bytes) => bytes,
            Err(_) => return Err(anyhow!("Unsupported key type!")),
        },
    };

    let keypair = Keypair::from_bytes(&secret_bytes)?;
    Ok(keypair)
}

pub fn parse_solana_config() -> Option<SolanaConfig> {
    let home = if cfg!(unix) {
        env::var_os("HOME").expect("Coulnd't find UNIX home key.")
    } else if cfg!(windows) {
        let drive = env::var_os("HOMEDRIVE").expect("Coulnd't find Windows home drive key.");
        let path = env::var_os("HOMEPATH").expect("Coulnd't find Windows home path key.");
        Path::new(&drive).join(&path).as_os_str().to_owned()
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME").expect("Coulnd't find MacOS home key.")
    } else {
        panic!("Unsupported OS!");
    };

    let config_path = Path::new(&home)
        .join(".config")
        .join("solana")
        .join("cli")
        .join("config.yml");

    let conf_file = match fs::File::open(config_path) {
        Ok(f) => f,
        Err(_) => return None,
    };
    serde_yaml::from_reader(&conf_file).ok()
}

fn convert_creator(c: &NFTCreator) -> Result<Creator> {
    Ok(Creator {
        address: Pubkey::from_str(&c.address)?,
        verified: c.verified,
        share: c.share,
    })
}

pub fn parse_creators(creators_json: &Value) -> Result<Vec<NFTCreator>> {
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
            .to_string();
        let share = creator
            .get("share")
            .ok_or(anyhow!("Invalid share!"))?
            .as_u64()
            .ok_or(anyhow!("Invalid share!"))? as u8;
        creators.push(NFTCreator {
            address,
            verified: false,
            share,
        });
    }
    Ok(creators)
}

pub fn parse_name(body: &Value) -> Result<String> {
    let name = body
        .get("name")
        .ok_or(anyhow!("Invalid name!"))?
        .as_str()
        .ok_or(anyhow!("Invalid name!"))?
        .to_string();
    Ok(name)
}

pub fn parse_symbol(body: &Value) -> Result<String> {
    let symbol = body
        .get("symbol")
        .ok_or(anyhow!("Invalid symbol!"))?
        .as_str()
        .ok_or(anyhow!("Invalid symbol!"))?
        .to_string();
    Ok(symbol)
}

pub fn parse_seller_fee_basis_points(body: &Value) -> Result<u16> {
    let seller_fee_basis_points =
        body.get("seller_fee_basis_points")
            .ok_or(anyhow!("Invalid seller_fee_basis_points!"))?
            .as_u64()
            .ok_or(anyhow!("Invalid seller_fee_basis_points!"))? as u16;
    Ok(seller_fee_basis_points)
}

pub fn convert_local_to_remote_data(local: NFTData) -> Result<Data> {
    let creators = local
        .creators
        .ok_or(anyhow!("No creators specified in json file!"))?
        .iter()
        .map(convert_creator)
        .collect::<Result<Vec<Creator>>>()?;

    let data = Data {
        name: local.name,
        symbol: local.symbol,
        uri: local.uri,
        seller_fee_basis_points: local.seller_fee_basis_points,
        creators: Some(creators),
    };
    Ok(data)
}

pub fn is_only_one_option<T, U>(option1: &Option<T>, option2: &Option<U>) -> bool {
    match (option1, option2) {
        (Some(_), None) | (None, Some(_)) => true,
        (Some(_), Some(_)) => false,
        (None, None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_white_space_keys() {
        // Arrange
        let whitespace_key_path = String::from("./tests/test_key_whitespace.txt");
        let newline_key_path = String::from("./tests/test_key_newline.txt");
        let phantom_key_path = String::from("./tests/test_key_phantom.txt");

        // Act
        let whitespace_res = parse_keypair(&whitespace_key_path);
        let newline_res = parse_keypair(&newline_key_path);
        let phantom_res = parse_keypair(&phantom_key_path);

        // Assert
        assert!(whitespace_res.is_ok());
        assert!(newline_res.is_ok());
        assert!(phantom_res.is_ok());
    }
}
