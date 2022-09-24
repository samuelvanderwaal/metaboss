use anyhow::{anyhow, Result};
use retry::{delay::Exponential, retry};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, transaction::Transaction,
};
use std::{ops::Add, sync::Arc};

use crate::data::FoundError;
use crate::wtf_errors::{
    ANCHOR_ERROR, AUCTIONEER_ERROR, AUCTION_HOUSE_ERROR, CANDY_ERROR, METADATA_ERROR,
};

pub fn send_and_confirm_transaction(
    client: &RpcClient,
    keypair: Keypair,
    instructions: &[Instruction],
) -> Result<String> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );

    let sig = res?;

    println!("TxId: {}", sig);
    Ok(sig.to_string())
}

pub async fn async_send_and_confirm_transaction(
    async_client: Arc<AsyncRpcClient>,
    keypair: Arc<Keypair>,
    instructions: &[Instruction],
) -> Result<String> {
    let recent_blockhash = async_client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&keypair.pubkey()),
        &[&*keypair],
        recent_blockhash,
    );

    let sig = async_client.send_and_confirm_transaction(&tx).await?;

    Ok(sig.to_string())
}

pub async fn retry_with_cache() {}

pub fn generate_phf_map_var(var_name: &str) -> String {
    format!("pub static {var_name}: phf::Map<&'static str, &'static str> = phf_map! {{\n")
}

pub fn convert_to_wtf_error(file_name: &str, file_contents: &str) -> Result<String> {
    let file_names = file_name.replace(".rs", "").replace('-', " ");
    let file_names_split = file_names.split(' ');

    let file_name_capitalized = file_names_split
        .clone()
        .into_iter()
        .map(|s| s.to_ascii_uppercase())
        .collect::<Vec<String>>()
        .join("_");

    let mut error_contents = generate_phf_map_var(&file_name_capitalized);

    let is_anchor = file_name.contains("anchor");

    let mut starting_error_number: i64 = match is_anchor {
        true => 100,
        false => match file_contents.contains("#[msg") {
            true => 6000,
            false => 0,
        },
    };

    let enum_name = if is_anchor {
        String::from("ErrorCode")
    } else {
        file_names_split
            .into_iter()
            .map(|s| {
                format!(
                    "{}{}",
                    s.get(0..1).unwrap().to_ascii_uppercase(),
                    s.get(1..).unwrap()
                )
            })
            .collect::<Vec<String>>()
            .join("")
    };

    let error_index = match file_contents.find(&enum_name) {
        Some(index) => index,
        None => return Err(anyhow!("Could not find Error enum")),
    };

    let trimmed_content = match file_contents.get(error_index.add(enum_name.len() + 2)..) {
        Some(contents) => contents.trim(),
        None => return Err(anyhow!("Malformed Error enum")),
    };

    let error_lines = match trimmed_content.contains('}') {
        true => trimmed_content.lines(),
        false => return Err(anyhow!("Malformed Error enum")),
    };

    let mut parsed_error_line = String::from("\",\n");

    for error_line in error_lines {
        let error_line = error_line.trim();

        if error_line.starts_with('}') {
            break;
        }

        if error_line.starts_with('/') || error_line.is_empty() {
            continue;
        } else if !error_line.starts_with("#[")
            && !error_line.starts_with('\"')
            && !error_line.ends_with('\"')
            && !error_line.ends_with(")]")
        {
            let enum_end_index = match error_line.find(',') {
                Some(index) => index,
                None => return Err(anyhow!("Malformed Error enum")),
            };

            let mut error_enum = match error_line.get(..enum_end_index) {
                Some(res) => res,
                None => return Err(anyhow!("Cannot parse Error enum")),
            };

            if error_enum.contains('=') {
                let error_code_combo = error_enum.split('=').collect::<Vec<&str>>();

                error_enum = error_code_combo[0].trim();
                starting_error_number = error_code_combo[1].trim().parse::<i64>()?;
            }

            parsed_error_line = format!(
                "    \"{:X}\" => \"{}{}",
                starting_error_number, error_enum, parsed_error_line
            );
        } else if error_line.starts_with("#[") && error_line.ends_with(")]") {
            let parsed_message = error_line
                .replace("#[", "")
                .replace("error(\"", "")
                .replace("msg(\"", "")
                .replace("\")]", "");

            parsed_error_line = format!(": {}\",\n", parsed_message);
        }

        if parsed_error_line.contains("=>") {
            error_contents.push_str(&parsed_error_line);
            starting_error_number += 1;
            parsed_error_line = String::from("\",\n");
        }
    }

    error_contents.push_str("};\n\n");
    Ok(error_contents)
}

pub fn find_errors(hex_code: &str) -> Vec<FoundError> {
    let hex_code = hex_code.to_uppercase();
    let mut found_errors: Vec<FoundError> = Vec::new();

    if let Some(e) = ANCHOR_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Anchor Program".to_string(),
            message: e.to_string(),
        });
    }

    if let Some(e) = METADATA_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Token Metadata".to_string(),
            message: e.to_string(),
        });
    }

    if let Some(e) = AUCTION_HOUSE_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Auction House".to_string(),
            message: e.to_string(),
        });
    }

    if let Some(e) = AUCTIONEER_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Auctioneer".to_string(),
            message: e.to_string(),
        });
    }

    if let Some(e) = CANDY_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Candy Machine".to_string(),
            message: e.to_string(),
        });
    }

    found_errors
}

pub fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}
