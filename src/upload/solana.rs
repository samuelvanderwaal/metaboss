use anyhow::{anyhow, Result};
use retry::delay::Exponential;
use retry::retry;
use solana_client::rpc_client::RpcClient;
use solana_json::instruction::SetValueArgs;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use std::fs::File;

use crate::update::{parse_keypair, parse_solana_config};

pub struct UploadSolanaArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub json_file: String,
    pub rate_limit: usize,
    pub retries: u8,
}

pub async fn upload_solana(args: UploadSolanaArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let f = File::open(args.json_file)?;
    let data: serde_json::Value = serde_json::from_reader(f)?;
    // println!("{:#?}", data);

    let json_map = match data {
        serde_json::Value::Object(map) => Ok(map),
        _ => Err("Expected JSON object"),
    }
    .map_err(|e| anyhow!(e))?;

    let json_account_keypair = Keypair::new();
    let json_metadata_account =
        solana_json::pda::find_metadata_account(&json_account_keypair.pubkey());

    let init_ix = solana_json::instruction::initialize(
        solana_json::ID,
        json_account_keypair.pubkey(),
        json_metadata_account.0,
        keypair.pubkey(),
    );

    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&keypair.pubkey()),
        &[&keypair, &json_account_keypair],
        args.client.get_latest_blockhash()?,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&init_tx),
    );

    let sig = res?;
    println!("Init Signature: {sig}");

    for (key, value) in json_map {
        // println!("{{{}: {}}}", key, value);
        let value = serde_json::json!({ key.clone(): value }).to_string();
        let set_ix = solana_json::instruction::set_value(
            solana_json::ID,
            json_account_keypair.pubkey(),
            json_metadata_account.0,
            keypair.pubkey(),
            SetValueArgs { value },
        );
        let set_tx = Transaction::new_signed_with_payer(
            &[set_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            args.client.get_latest_blockhash()?,
        );
        // Send tx with retries.
        let res = retry(
            Exponential::from_millis_with_factor(250, 2.0).take(3),
            || args.client.send_and_confirm_transaction(&set_tx),
        );
        let sig = res?;
        println!("Set {key} Signature: {sig}");
    }

    println!("JSON Account: {}", json_account_keypair.pubkey());

    // println!("{:#?}", keypair);
    Ok(())
}
