use anyhow::{anyhow, Context, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use retry::{delay::Exponential, retry};
use serde::Deserialize;
use serde_json::json;
use solana_client::rpc_request::RpcRequest;
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_program::instruction::AccountMeta;
use solana_program::program_pack::Pack;
use solana_program::system_program;
use solana_program::{pubkey, pubkey::Pubkey};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, transaction::Transaction,
};
use spl_token::state::Account;
use std::str::FromStr;
use std::{ops::Add, sync::Arc};

use crate::data::FoundError;
use crate::wtf_errors::{
    ANCHOR_ERROR, AUCTIONEER_ERROR, AUCTION_HOUSE_ERROR, CANDY_CORE_ERROR, CANDY_ERROR,
    CANDY_GUARD_ERROR, METADATA_ERROR,
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

    println!("Tx sig: {sig}");
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
    } else if file_name_capitalized == "CANDY_CORE_ERROR" {
        String::from("CandyError")
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

            parsed_error_line =
                format!("    \"{starting_error_number:X}\" => \"{error_enum}{parsed_error_line}");
        } else if error_line.starts_with("#[") && error_line.ends_with(")]") {
            let parsed_message = error_line
                .replace("#[", "")
                .replace("error(\"", "")
                .replace("msg(\"", "")
                .replace("\")]", "");

            parsed_error_line = format!(": {parsed_message}\",\n");
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

    if let Some(e) = CANDY_CORE_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Candy Core".to_string(),
            message: e.to_string(),
        });
    }

    if let Some(e) = CANDY_GUARD_ERROR.get(&hex_code).cloned() {
        found_errors.push(FoundError {
            domain: "Candy Guard".to_string(),
            message: e.to_string(),
        });
    }

    found_errors
}

pub fn find_tm_error(hex_code: &str) -> Option<String> {
    let hex_code = hex_code.to_uppercase();

    METADATA_ERROR.get(&hex_code).map(|e| e.to_string())
}

pub fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_bytes(&keypair.to_bytes()).unwrap()
}

pub fn get_largest_token_account_owner(client: &RpcClient, mint: Pubkey) -> Result<Pubkey> {
    let request = RpcRequest::Custom {
        method: "getTokenLargestAccounts",
    };
    let params = json!([mint.to_string(), { "commitment": "confirmed" }]);
    let result: JRpcResponse = client
        .send(request, params)
        .context("Failed to get largest token accounts from RPC")?;

    let token_accounts: Vec<TokenAccount> = result
        .value
        .into_iter()
        .filter(|account| account.amount.parse::<u64>().unwrap_or(0) == 1)
        .collect();

    if token_accounts.len() > 1 {
        return Err(anyhow!(
            "Mint account {} had more than one token account with 1 token",
            mint
        ));
    }

    if token_accounts.is_empty() {
        return Err(anyhow!(
            "Mint account {} had zero token accounts with 1 token",
            mint
        ));
    }

    let token_account = Pubkey::from_str(&token_accounts[0].address).map_err(|_| {
        anyhow!(
            "Invalid token account address: {}",
            token_accounts[0].address
        )
    })?;

    let account = client
        .get_account_with_commitment(&token_account, CommitmentConfig::confirmed())
        .context(format!(
            "Failed to get account data for token account {}",
            token_account
        ))?
        .value
        .ok_or_else(|| anyhow!("Token account {} not found on-chain", token_account))?;

    let account_data = Account::unpack(&account.data)
        .context("Failed to unpack token account data (SPL Token format)")?;

    Ok(account_data.owner)
}

#[derive(Debug, Deserialize)]
pub struct JRpcResponse {
    value: Vec<TokenAccount>,
}

#[derive(Debug, Deserialize)]
struct TokenAccount {
    address: String,
    amount: String,
    // decimals: u8,
    // #[serde(rename = "uiAmount")]
    // ui_amount: f32,
    // #[serde(rename = "uiAmountString")]
    // ui_amount_string: String,
}

const MPL_TOOLBOX_ID: Pubkey = pubkey!("TokExjvjJmhKaRBShsBAsbSvEWMA1AgUNK7ps4SAc2p");

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
pub enum TokenExtrasInstruction {
    /// Creates a new associated token account for the given mint and owner, if and only if
    /// the given token account does not exists and the token account is the same as the
    /// associated token account. That way, clients can ensure that, after this instruction,
    /// the token account will exists.
    ///
    /// Notice this instruction asks for both the token account and the associated token account (ATA)
    /// These may or may not be the same account. Here are all the possible cases:
    ///
    /// - Token exists and Token is ATA: Instruction succeeds.
    /// - Token exists and Token is not ATA: Instruction succeeds.
    /// - Token does not exist and Token is ATA: Instruction creates the ATA account and succeeds.
    /// - Token does not exist and Token is not ATA: Instruction fails as we cannot create a
    ///   non-ATA account without it being a signer.
    ///
    /// Note that additional checks are made to ensure that the token account provided
    /// matches the mint account and owner account provided.
    CreateTokenIfMissing,
}

pub fn create_token_if_missing_instruction(
    payer: &Pubkey,
    token: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    ata: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: MPL_TOOLBOX_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*token, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new(*ata, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: TokenExtrasInstruction::CreateTokenIfMissing
            .try_to_vec()
            .unwrap(),
    }
}

pub fn create_token_22_if_missing_instruction(
    payer: &Pubkey,
    token: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    ata: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: MPL_TOOLBOX_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*token, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new(*ata, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: TokenExtrasInstruction::CreateTokenIfMissing
            .try_to_vec()
            .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── find_errors ──────────────────────────────────────────────────

    #[test]
    fn find_errors_returns_anchor_error_for_known_hex_code() {
        // "64" is InstructionMissing in ANCHOR_ERROR
        let results = find_errors("64");
        assert!(!results.is_empty(), "Expected at least one result for '64'");
        let anchor_hit = results.iter().find(|e| e.domain == "Anchor Program");
        assert!(
            anchor_hit.is_some(),
            "Expected an Anchor Program error for hex code '64'"
        );
        assert!(
            anchor_hit.unwrap().message.contains("InstructionMissing"),
            "Expected message to contain 'InstructionMissing'"
        );
    }

    #[test]
    fn find_errors_returns_metadata_error_for_known_hex_code() {
        // "0" is InstructionUnpackError in METADATA_ERROR
        let results = find_errors("0");
        let meta_hit = results.iter().find(|e| e.domain == "Token Metadata");
        assert!(
            meta_hit.is_some(),
            "Expected a Token Metadata error for hex code '0'"
        );
        assert!(
            meta_hit.unwrap().message.contains("InstructionUnpackError"),
            "Expected message to contain 'InstructionUnpackError'"
        );
    }

    #[test]
    fn find_errors_is_case_insensitive() {
        let lower = find_errors("64");
        let upper = find_errors("64"); // already uppercase, but test lowercase hex
        assert_eq!(lower.len(), upper.len());

        // Try with a hex code that has alphabetic chars if available;
        // at minimum verify lowercasing path works
        let results = find_errors("a");
        let results_upper = find_errors("A");
        assert_eq!(results.len(), results_upper.len());
    }

    #[test]
    fn find_errors_returns_empty_for_unknown_code() {
        let results = find_errors("ZZZZZ");
        assert!(
            results.is_empty(),
            "Expected no results for unknown hex code"
        );
    }

    #[test]
    fn find_errors_returns_empty_for_empty_string() {
        let results = find_errors("");
        assert!(results.is_empty(), "Expected no results for empty hex code");
    }

    // ── find_tm_error ────────────────────────────────────────────────

    #[test]
    fn find_tm_error_returns_some_for_known_code() {
        let result = find_tm_error("0");
        assert!(result.is_some(), "Expected Some for known code '0'");
        assert!(
            result.unwrap().contains("InstructionUnpackError"),
            "Expected message to contain 'InstructionUnpackError'"
        );
    }

    #[test]
    fn find_tm_error_returns_some_for_another_known_code() {
        // "1" is InstructionPackError
        let result = find_tm_error("1");
        assert!(result.is_some(), "Expected Some for known code '1'");
        assert!(
            result.unwrap().contains("InstructionPackError"),
            "Expected message to contain 'InstructionPackError'"
        );
    }

    #[test]
    fn find_tm_error_is_case_insensitive() {
        let lower = find_tm_error("a");
        let upper = find_tm_error("A");
        assert_eq!(lower, upper);
    }

    #[test]
    fn find_tm_error_returns_none_for_unknown_code() {
        let result = find_tm_error("ZZZZZ");
        assert!(result.is_none(), "Expected None for unknown hex code");
    }

    #[test]
    fn find_tm_error_returns_none_for_empty_string() {
        let result = find_tm_error("");
        assert!(result.is_none(), "Expected None for empty hex code");
    }

    // ── generate_phf_map_var ─────────────────────────────────────────

    #[test]
    fn generate_phf_map_var_produces_correct_format() {
        let output = generate_phf_map_var("MY_ERRORS");
        assert_eq!(
            output,
            "pub static MY_ERRORS: phf::Map<&'static str, &'static str> = phf_map! {\n"
        );
    }

    #[test]
    fn generate_phf_map_var_handles_empty_name() {
        let output = generate_phf_map_var("");
        assert!(output.starts_with("pub static : phf::Map"));
    }

    #[test]
    fn generate_phf_map_var_preserves_casing() {
        let output = generate_phf_map_var("mixedCase_Var");
        assert!(output.contains("mixedCase_Var"));
    }

    // ── convert_to_wtf_error ─────────────────────────────────────────

    #[test]
    fn convert_to_wtf_error_parses_simple_enum() {
        let file_name = "my-error.rs";
        let file_contents = r#"
pub enum MyError {
    #[error("Something went wrong")]
    SomethingWrong,
    #[error("Another error")]
    AnotherError,
}
"#;
        let result = convert_to_wtf_error(file_name, file_contents).unwrap();
        assert!(
            result.contains("pub static MY_ERROR: phf::Map"),
            "Expected PHF map header"
        );
        assert!(
            result.contains("SomethingWrong"),
            "Expected first variant name"
        );
        assert!(
            result.contains("AnotherError"),
            "Expected second variant name"
        );
        assert!(
            result.contains("Something went wrong"),
            "Expected first error message"
        );
        assert!(result.ends_with("};\n\n"), "Expected closing braces");
    }

    #[test]
    fn convert_to_wtf_error_parses_anchor_style_errors() {
        let file_name = "anchor-error.rs";
        let file_contents = r#"
pub enum ErrorCode {
    #[msg("Invalid account")]
    InvalidAccount,
    #[msg("Unauthorized")]
    Unauthorized,
}
"#;
        let result = convert_to_wtf_error(file_name, file_contents).unwrap();
        assert!(result.contains("ANCHOR_ERROR"));
        // Anchor errors start at 100 (0x64)
        assert!(
            result.contains("\"64\""),
            "Expected hex code 64 for first anchor error"
        );
        assert!(result.contains("InvalidAccount: Invalid account"));
        assert!(result.contains("Unauthorized: Unauthorized"));
    }

    #[test]
    fn convert_to_wtf_error_parses_msg_style_starting_at_6000() {
        let file_name = "test-error.rs";
        let file_contents = r#"
pub enum TestError {
    #[msg("First error")]
    FirstError,
    #[msg("Second error")]
    SecondError,
}
"#;
        let result = convert_to_wtf_error(file_name, file_contents).unwrap();
        // #[msg] style, non-anchor -> starts at 6000 (0x1770)
        assert!(
            result.contains("\"1770\""),
            "Expected hex code 1770 for error starting at 6000"
        );
        assert!(result.contains("FirstError: First error"));
    }

    #[test]
    fn convert_to_wtf_error_handles_explicit_error_codes() {
        let file_name = "custom-error.rs";
        let file_contents = r#"
pub enum CustomError {
    #[error("Start error")]
    StartError = 42,
    #[error("Next error")]
    NextError,
}
"#;
        let result = convert_to_wtf_error(file_name, file_contents).unwrap();
        // 42 = 0x2A
        assert!(
            result.contains("\"2A\""),
            "Expected hex code 2A for explicit code 42"
        );
        // Next should be 43 = 0x2B
        assert!(
            result.contains("\"2B\""),
            "Expected hex code 2B for next error after 42"
        );
    }

    #[test]
    fn convert_to_wtf_error_returns_error_when_enum_not_found() {
        let file_name = "missing-error.rs";
        let file_contents = "pub struct NotAnEnum { field: u8 }";
        let result = convert_to_wtf_error(file_name, file_contents);
        assert!(result.is_err(), "Expected error when enum is not found");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Could not find Error enum"),);
    }

    #[test]
    fn convert_to_wtf_error_returns_error_for_malformed_enum_no_closing_brace() {
        let file_name = "bad-error.rs";
        let file_contents = r#"
pub enum BadError {
    #[error("Oops")]
    Oops,
"#;
        let result = convert_to_wtf_error(file_name, file_contents);
        assert!(
            result.is_err(),
            "Expected error for malformed enum without closing brace"
        );
    }

    #[test]
    fn convert_to_wtf_error_capitalizes_multiword_file_name() {
        let file_name = "candy-core-error.rs";
        // candy-core-error special-cases the enum name to CandyError
        let file_contents = r#"
pub enum CandyError {
    #[error("Bad candy")]
    BadCandy,
}
"#;
        let result = convert_to_wtf_error(file_name, file_contents).unwrap();
        assert!(
            result.contains("CANDY_CORE_ERROR"),
            "Expected uppercased variable name"
        );
    }

    // ── clone_keypair ────────────────────────────────────────────────

    #[test]
    fn clone_keypair_produces_identical_keypair() {
        let original = Keypair::new();
        let cloned = clone_keypair(&original);
        assert_eq!(
            original.to_bytes(),
            cloned.to_bytes(),
            "Cloned keypair bytes must match original"
        );
        assert_eq!(
            original.pubkey(),
            cloned.pubkey(),
            "Cloned keypair pubkey must match original"
        );
    }

    #[test]
    fn clone_keypair_produces_independent_instance() {
        let original = Keypair::new();
        let cloned = clone_keypair(&original);
        // They should be distinct allocations but equal in value
        let orig_ptr = &original as *const Keypair;
        let clone_ptr = &cloned as *const Keypair;
        assert_ne!(
            orig_ptr, clone_ptr,
            "Cloned keypair should be a separate allocation"
        );
        assert_eq!(original.to_bytes(), cloned.to_bytes());
    }
}
