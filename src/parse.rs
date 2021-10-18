use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use solana_sdk::signer::keypair::Keypair;
use std::{env, fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
pub struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

pub fn parse_keypair(path: &String) -> Result<Keypair> {
    let secret_string = fs::read_to_string(path).context("Can't find key file")?;
    let secret_bytes: Vec<u8> = serde_json::from_str(&secret_string)?;

    let keypair = Keypair::from_bytes(&secret_bytes)?;
    Ok(keypair)
}

pub fn parse_solana_config() -> Option<SolanaConfig> {
    let key = "HOME";
    let home = match env::var_os(key) {
        Some(val) => val,
        None => return None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_white_space_keys() {
        // Arrange
        let whitespace_key_path = String::from("./test/test_key_whitespace.txt");
        let newline_key_path = String::from("./test/test_key_newline.txt");

        // Act
        let whitespace_res = parse_keypair(&whitespace_key_path);
        let newline_res = parse_keypair(&newline_key_path);

        // Assert
        assert!(whitespace_res.is_ok());
        assert!(newline_res.is_ok());
    }
}
