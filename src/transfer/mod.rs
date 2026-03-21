use std::str::FromStr;

use anyhow::Result;
use metaboss_lib::data::{Asset, Priority};
use mpl_token_metadata::{
    instructions::TransferV1Builder,
    types::{ProgrammableConfig, TokenStandard},
};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{compute_budget::ComputeBudgetInstruction, signer::Signer};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    parse::{parse_keypair, parse_solana_config},
    utils::send_and_confirm_transaction,
};

/// The program ID for the mpl-token-auth-rules program.
const MPL_TOKEN_AUTH_RULES_ID: Pubkey =
    solana_program::pubkey!("auth9SigNpDKz4sJJ1DfCTuZrZNSAgh9sFD3rboVmgg");

pub fn process_transfer_asset(
    client: &RpcClient,
    keypair_path: Option<String>,
    receiver: String,
    receiver_account: Option<String>,
    mint: String,
    amount: u64,
    priority: Priority,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    // Authority is the payer as well.
    let authority = parse_keypair(keypair_path, solana_opts);
    let receiver = Pubkey::from_str(&receiver)?;
    let mint = Pubkey::from_str(&mint)?;

    let source_ata = get_associated_token_address(&authority.pubkey(), &mint);
    let destination_token = if let Some(account) = receiver_account {
        Pubkey::from_str(&account)?
    } else {
        get_associated_token_address(&receiver, &mint)
    };

    let mut asset = Asset::new(mint);

    let mut transfer_builder = TransferV1Builder::new();
    transfer_builder
        .payer(authority.pubkey())
        .authority(authority.pubkey())
        .token(source_ata)
        .token_owner(authority.pubkey())
        .destination_token(destination_token)
        .destination_owner(receiver)
        .mint(asset.mint)
        .metadata(asset.metadata)
        .amount(amount);

    let md = asset.get_metadata(client)?;

    if matches!(
        md.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        // Always need the token records for pNFTs.
        let source_token_record = asset.get_token_record(&source_ata);
        let destination_token_record = asset.get_token_record(&destination_token);
        transfer_builder
            .token_record(Some(source_token_record))
            .destination_token_record(Some(destination_token_record));

        // If the asset's metadata account has auth rules set, we need to pass the
        // account in.
        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(auth_rules),
        }) = md.programmable_config
        {
            transfer_builder.authorization_rules_program(Some(MPL_TOKEN_AUTH_RULES_ID));
            transfer_builder.authorization_rules(Some(auth_rules));
        }
    }

    if matches!(
        md.token_standard,
        Some(
            TokenStandard::NonFungible
                | TokenStandard::NonFungibleEdition
                | TokenStandard::ProgrammableNonFungible
        ) | None
    ) {
        asset.add_edition();
        transfer_builder.edition(asset.edition);
    }

    let transfer_ix = transfer_builder.instruction();

    let micro_lamports = match priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_price(micro_lamports);

    let sig = send_and_confirm_transaction(client, authority, &[compute_budget_ix, transfer_ix])?;

    println!("Transferred asset: {mint:?}");
    println!("Transaction signature: {sig}");

    Ok(())
}
