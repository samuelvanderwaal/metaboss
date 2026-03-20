use anyhow::{anyhow, Context, Result};
use mpl_token_metadata::types::Creator;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::Keypair;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::{env, fs, path::Path, str::FromStr};

use crate::constants::ERROR_FILE_BEGIN;
use crate::utils::{convert_to_wtf_error, find_errors};

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

pub fn creator_is_verified(creators_opt: &Option<Vec<Creator>>, position: usize) -> bool {
    // Only add mints with a verified creator.
    if let Some(creators) = creators_opt {
        if creators[position].verified {
            return true;
        }
    }
    false
}

pub fn parse_solana_config() -> Option<SolanaConfig> {
    let home = if cfg!(unix) {
        env::var_os("HOME").expect("Coulnd't find UNIX home key.")
    } else if cfg!(windows) {
        let drive = env::var_os("HOMEDRIVE").expect("Could not find Windows home drive key.");
        let path = env::var_os("HOMEPATH").expect("Could not find Windows home path key.");
        Path::new(&drive).join(path).as_os_str().to_owned()
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

pub fn parse_keypair(
    keypair_opt: Option<String>,
    sol_config_option: Option<SolanaConfig>,
) -> Keypair {
    let keypair = match keypair_opt {
        Some(keypair_path) => read_keypair(&keypair_path).expect("Failed to read keypair file."),
        None => match sol_config_option {
            Some(ref sol_config) => {
                read_keypair(&sol_config.keypair_path).expect("Failed to read keypair file.")
            }
            None => read_keypair(&(*shellexpand::tilde("~/.config/solana/id.json")).to_string())
                .expect("Failed to read keypair file."),
        },
    };
    keypair
}

pub fn read_keypair(path: &String) -> Result<Keypair> {
    let secret_string: String = fs::read_to_string(path).context("Can't find key file")?;

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

pub fn parse_creators(creators_json: &Value) -> Result<Vec<Creator>> {
    let mut creators = Vec::new();

    for creator in creators_json
        .as_array()
        .ok_or_else(|| anyhow!("Invalid creators array!"))?
    {
        let address = creator
            .get("address")
            .ok_or_else(|| anyhow!("Invalid address!"))?
            .as_str()
            .ok_or_else(|| anyhow!("Invalid address!"))?
            .to_string();
        let share = creator
            .get("share")
            .ok_or_else(|| anyhow!("Invalid share!"))?
            .as_u64()
            .ok_or_else(|| anyhow!("Invalid share!"))? as u8;
        creators.push(Creator {
            address: Pubkey::from_str(&address)?,
            verified: false,
            share,
        });
    }
    Ok(creators)
}

pub fn parse_name(body: &Value) -> Result<String> {
    let name = body
        .get("name")
        .ok_or_else(|| anyhow!("Invalid name!"))?
        .as_str()
        .ok_or_else(|| anyhow!("Invalid name!"))?
        .to_string();
    Ok(name)
}

pub fn parse_symbol(body: &Value) -> Result<String> {
    let symbol = body
        .get("symbol")
        .ok_or_else(|| anyhow!("Invalid symbol!"))?
        .as_str()
        .ok_or_else(|| anyhow!("Invalid symbol!"))?
        .to_string();
    Ok(symbol)
}

pub fn parse_seller_fee_basis_points(body: &Value) -> Result<u16> {
    let seller_fee_basis_points =
        body.get("seller_fee_basis_points")
            .ok_or_else(|| anyhow!("Invalid seller_fee_basis_points!"))?
            .as_u64()
            .ok_or_else(|| anyhow!("Invalid seller_fee_basis_points!"))? as u16;
    Ok(seller_fee_basis_points)
}

pub fn is_only_one_option<T, U>(option1: &Option<T>, option2: &Option<U>) -> bool {
    match (option1, option2) {
        (Some(_), None) | (None, Some(_)) => true,
        (Some(_), Some(_)) => false,
        (None, None) => false,
    }
}

pub fn parse_cli_creators(new_creators: String, should_append: bool) -> Result<Vec<Creator>> {
    let mut creators = Vec::new();

    for nc in new_creators.split(',') {
        let mut c = nc.split(':');
        let address = c.next().ok_or_else(|| anyhow!("Missing address!"))?;
        let address = Pubkey::from_str(address)
            .map_err(|_| anyhow!(format!("Invalid creator address: {address:?}!")))?;
        let share = if should_append {
            c.next();
            0u8
        } else {
            c.next()
                .ok_or_else(|| anyhow!("Invalid creator share, must be 0-100!"))?
                .parse::<u8>()?
        };
        let verified = c
            .next()
            .ok_or_else(|| anyhow!("Missing creator verified: must be 'true' or 'false'!"))?
            .parse::<bool>()?;
        creators.push(Creator {
            address,
            share,
            verified,
        });
    }

    if creators.len() > 5 {
        return Err(anyhow!("Too many creators: maximum of five!"));
    }

    Ok(creators)
}

pub fn parse_errors_file() -> Result<()> {
    let wtf_error_path = Path::new("src/wtf_errors.rs");
    let error_files_dir = PathBuf::from("src/error_files");

    match error_files_dir.read_dir() {
        Ok(files) => {
            let mut error_file_content = String::from(ERROR_FILE_BEGIN);
            for file in files.flatten() {
                let file_name = file.file_name();
                let file_contents = read_to_string(file.path())?;
                let error_content =
                    convert_to_wtf_error(file_name.to_str().unwrap(), &file_contents)?;
                error_file_content.push_str(&error_content);
            }
            fs::write(wtf_error_path, error_file_content)?;
            Ok(())
        }
        Err(_) => Err(anyhow!("Error folder doesn't exist")),
    }
}

pub fn parse_errors_code(error_code: &str) -> Result<()> {
    let parsed_error_code = if error_code.contains("0x") {
        error_code.replace("0x", "")
    } else {
        format!("{:X}", error_code.parse::<i64>()?)
    };

    let errors = find_errors(&parsed_error_code);

    if errors.is_empty() {
        return Err(anyhow!("Invalid Error Code"));
    }

    for error in errors {
        println!("\t{:<10} |\t{}", error.domain, error.message);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_white_space_keys() {
        // Arrange
        let whitespace_key_path = String::from("./tests/test_key_whitespace.txt");
        let newline_key_path = String::from("./tests/test_key_newline.txt");
        let phantom_key_path = String::from("./tests/test_key_phantom.txt");

        // Act
        let whitespace_res = read_keypair(&whitespace_key_path);
        let newline_res = read_keypair(&newline_key_path);
        let phantom_res = read_keypair(&phantom_key_path);

        // Assert
        assert!(whitespace_res.is_ok());
        assert!(newline_res.is_ok());
        assert!(phantom_res.is_ok());
    }

    // -- creator_is_verified tests --

    #[test]
    fn test_creator_is_verified_true() {
        // Arrange
        let creators = Some(vec![Creator {
            address: Pubkey::new_unique(),
            verified: true,
            share: 100,
        }]);

        // Act & Assert
        assert!(creator_is_verified(&creators, 0));
    }

    #[test]
    fn test_creator_is_verified_false() {
        // Arrange
        let creators = Some(vec![Creator {
            address: Pubkey::new_unique(),
            verified: false,
            share: 100,
        }]);

        // Act & Assert
        assert!(!creator_is_verified(&creators, 0));
    }

    #[test]
    fn test_creator_is_verified_none() {
        // Act & Assert
        assert!(!creator_is_verified(&None, 0));
    }

    #[test]
    #[should_panic]
    fn test_creator_is_verified_out_of_bounds() {
        // Arrange
        let creators = Some(vec![Creator {
            address: Pubkey::new_unique(),
            verified: true,
            share: 100,
        }]);

        // Act - should panic on out-of-bounds index
        creator_is_verified(&creators, 5);
    }

    // -- parse_creators tests --

    #[test]
    fn test_parse_creators_valid_single_creator() {
        // Arrange
        let pubkey = Pubkey::new_unique();
        let creators_json = json!([
            {"address": pubkey.to_string(), "share": 100}
        ]);

        // Act
        let result = parse_creators(&creators_json).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].address, pubkey);
        assert_eq!(result[0].share, 100);
    }

    #[test]
    fn test_parse_creators_valid_multiple_creators() {
        // Arrange
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();
        let creators_json = json!([
            {"address": pubkey1.to_string(), "share": 60},
            {"address": pubkey2.to_string(), "share": 40}
        ]);

        // Act
        let result = parse_creators(&creators_json).unwrap();

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].share, 60);
        assert_eq!(result[1].share, 40);
    }

    #[test]
    fn test_parse_creators_all_verified_false() {
        // Arrange
        let pubkey = Pubkey::new_unique();
        let creators_json = json!([
            {"address": pubkey.to_string(), "share": 100}
        ]);

        // Act
        let result = parse_creators(&creators_json).unwrap();

        // Assert
        assert!(!result[0].verified);
    }

    #[test]
    fn test_parse_creators_error_not_array() {
        // Arrange
        let creators_json = json!({"address": "test", "share": 100});

        // Act
        let result = parse_creators(&creators_json);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_creators_error_missing_address() {
        // Arrange
        let creators_json = json!([{"share": 100}]);

        // Act
        let result = parse_creators(&creators_json);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_creators_error_missing_share() {
        // Arrange
        let pubkey = Pubkey::new_unique();
        let creators_json = json!([{"address": pubkey.to_string()}]);

        // Act
        let result = parse_creators(&creators_json);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_creators_error_invalid_pubkey() {
        // Arrange
        let creators_json = json!([{"address": "not_a_valid_pubkey", "share": 100}]);

        // Act
        let result = parse_creators(&creators_json);

        // Assert
        assert!(result.is_err());
    }

    // -- parse_name tests --

    #[test]
    fn test_parse_name_valid() {
        // Arrange
        let body = json!({"name": "My NFT"});

        // Act
        let result = parse_name(&body).unwrap();

        // Assert
        assert_eq!(result, "My NFT");
    }

    #[test]
    fn test_parse_name_missing() {
        // Arrange
        let body = json!({"symbol": "NFT"});

        // Act & Assert
        assert!(parse_name(&body).is_err());
    }

    #[test]
    fn test_parse_name_not_string() {
        // Arrange
        let body = json!({"name": 123});

        // Act & Assert
        assert!(parse_name(&body).is_err());
    }

    // -- parse_symbol tests --

    #[test]
    fn test_parse_symbol_valid() {
        // Arrange
        let body = json!({"symbol": "NFT"});

        // Act
        let result = parse_symbol(&body).unwrap();

        // Assert
        assert_eq!(result, "NFT");
    }

    #[test]
    fn test_parse_symbol_missing() {
        // Arrange
        let body = json!({"name": "test"});

        // Act & Assert
        assert!(parse_symbol(&body).is_err());
    }

    // -- parse_seller_fee_basis_points tests --

    #[test]
    fn test_parse_seller_fee_basis_points_valid() {
        // Arrange
        let body = json!({"seller_fee_basis_points": 500});

        // Act
        let result = parse_seller_fee_basis_points(&body).unwrap();

        // Assert
        assert_eq!(result, 500);
    }

    #[test]
    fn test_parse_seller_fee_basis_points_missing() {
        // Arrange
        let body = json!({"name": "test"});

        // Act & Assert
        assert!(parse_seller_fee_basis_points(&body).is_err());
    }

    #[test]
    fn test_parse_seller_fee_basis_points_not_number() {
        // Arrange
        let body = json!({"seller_fee_basis_points": "five hundred"});

        // Act & Assert
        assert!(parse_seller_fee_basis_points(&body).is_err());
    }

    // -- is_only_one_option tests --

    #[test]
    fn test_is_only_one_option_some_none() {
        assert!(is_only_one_option(&Some(1), &None::<i32>));
    }

    #[test]
    fn test_is_only_one_option_none_some() {
        assert!(is_only_one_option(&None::<i32>, &Some(1)));
    }

    #[test]
    fn test_is_only_one_option_some_some() {
        assert!(!is_only_one_option(&Some(1), &Some(2)));
    }

    #[test]
    fn test_is_only_one_option_none_none() {
        assert!(!is_only_one_option(&None::<i32>, &None::<i32>));
    }

    // -- parse_cli_creators tests --

    #[test]
    fn test_parse_cli_creators_single() {
        // Arrange
        let pubkey = Pubkey::new_unique();
        let input = format!("{}:100:true", pubkey);

        // Act
        let result = parse_cli_creators(input, false).unwrap();

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].address, pubkey);
        assert_eq!(result[0].share, 100);
        assert!(result[0].verified);
    }

    #[test]
    fn test_parse_cli_creators_multiple() {
        // Arrange
        let pk1 = Pubkey::new_unique();
        let pk2 = Pubkey::new_unique();
        let input = format!("{}:60:true,{}:40:false", pk1, pk2);

        // Act
        let result = parse_cli_creators(input, false).unwrap();

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].share, 60);
        assert_eq!(result[1].share, 40);
        assert!(result[0].verified);
        assert!(!result[1].verified);
    }

    #[test]
    fn test_parse_cli_creators_too_many() {
        // Arrange - 6 creators (over the limit of 5)
        let creators: Vec<String> = (0..6)
            .map(|_| format!("{}:10:true", Pubkey::new_unique()))
            .collect();
        let input = creators.join(",");

        // Act & Assert
        assert!(parse_cli_creators(input, false).is_err());
    }

    #[test]
    fn test_parse_cli_creators_append_sets_share_zero() {
        // Arrange
        let pubkey = Pubkey::new_unique();
        let input = format!("{}:100:true", pubkey);

        // Act
        let result = parse_cli_creators(input, true).unwrap();

        // Assert
        assert_eq!(result[0].share, 0);
    }

    #[test]
    fn test_parse_cli_creators_invalid_address() {
        // Arrange
        let input = "not_a_valid_address:100:true".to_string();

        // Act & Assert
        assert!(parse_cli_creators(input, false).is_err());
    }

    #[test]
    fn test_parse_cli_creators_missing_fields() {
        // Arrange - missing verified field
        let pubkey = Pubkey::new_unique();
        let input = format!("{}:100", pubkey);

        // Act & Assert
        assert!(parse_cli_creators(input, false).is_err());
    }
}
