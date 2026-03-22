#![allow(dead_code)]
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey,
};
use solana_sdk_ids::system_program;

use borsh::{BorshDeserialize, BorshSerialize};

use super::*;

pub struct AirdropSplArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub recipient_list: Option<String>,
    pub cache_file: Option<String>,
    pub mint: Pubkey,
    pub mint_tokens: bool,
    pub priority: Priority,
    pub rate_limit: Option<u64>,
}

pub async fn airdrop_spl(_args: AirdropSplArgs) -> Result<()> {
    anyhow::bail!("Airdrop SPL is temporarily unavailable during Solana v2 migration. The jib dependency needs to be updated.")
}

const MPL_TOOLBOX_ID: Pubkey = pubkey!("TokExjvjJmhKaRBShsBAsbSvEWMA1AgUNK7ps4SAc2p");

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
pub enum TokenExtrasInstruction {
    CreateTokenIfMissing,
}

fn create_token_if_missing_instruction(
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

// Decimals is max 9, so this shouldn't lose precision.
fn convert_to_base_units(amount: f64, decimals: u8) -> Option<u64> {
    let multiplier = 10u64.pow(decimals as u32);
    let base_units = (amount * multiplier as f64).round();

    if base_units > u64::MAX as f64 || base_units < 0.0 {
        None
    } else {
        Some(base_units as u64)
    }
}
