use anyhow::{Context, Result};
use solana_sdk::signer::keypair::Keypair;
use std::fs;

pub fn parse_keypair(path: &String) -> Result<Keypair> {
    let secret: Vec<u8> = fs::read_to_string(path)
        .context("Can't find key file")?
        .trim_start_matches("[")
        .trim_end_matches("]")
        .split(",")
        .map(|c| c.parse().unwrap())
        .collect();
    let keypair = Keypair::from_bytes(&secret)?;
    Ok(keypair)
}
