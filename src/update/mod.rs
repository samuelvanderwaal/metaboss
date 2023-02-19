pub mod creator;
pub mod data;
pub mod immutable;
pub mod name;
pub mod primary_sale_happened;
pub mod rule_set;
pub mod seller_fee_basis_points;
pub mod symbol;
pub mod token_standard;
pub mod update_authority;
pub mod uri;
pub mod uses;

pub use creator::*;
pub use data::*;
pub use immutable::*;
pub use name::*;
pub use primary_sale_happened::*;
pub use rule_set::*;
pub use seller_fee_basis_points::*;
pub use symbol::*;
pub use token_standard::*;
pub use update_authority::*;
pub use uri::*;
pub use uses::*;

pub use anyhow::{anyhow, Result as AnyResult};
pub use async_trait::async_trait;
pub use log::{debug, error, info, warn};
pub use metaboss_lib::{
    decode::{decode_metadata_from_mint, ToPubkey},
    derive::{derive_edition_pda, derive_metadata_pda},
    nft::get_nft_token_account,
    update::{update_asset, UpdateAssetArgs},
};
pub use mpl_token_metadata::{
    instruction::{set_token_standard, update_metadata_accounts_v2, RuleSetToggle, UpdateArgs},
    pda::find_token_record_account,
    state::{DataV2, Metadata, ProgrammableConfig, TokenStandard},
    ID as TOKEN_METADATA_PROGRAM_ID,
};

pub use solana_client::rpc_client::RpcClient;
pub use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};
pub use spl_token::state::Account as TokenAccount;
pub use std::{cmp, fmt::Display, str::FromStr, sync::Arc};

pub use crate::cache::{Action, BatchActionArgs, RunActionArgs};
pub use crate::decode::{decode, get_metadata_pda};
pub use crate::errors::ActionError;
pub use crate::parse::parse_solana_config;
pub use crate::parse::{parse_cli_creators, parse_keypair};

pub fn update_asset_preface(
    client: &RpcClient,
    mint_account: &str,
) -> AnyResult<(Metadata, Option<Pubkey>, Option<Pubkey>)> {
    let current_md = decode_metadata_from_mint(client, mint_account)
        .map_err(|e| ActionError::ActionFailed(mint_account.to_string(), e.to_string()))?;

    // We need the token account passed in for pNFT updates.
    let token = if let Some(TokenStandard::ProgrammableNonFungible) = current_md.token_standard {
        Some(
            get_nft_token_account(client, mint_account)
                .map_err(|e| ActionError::ActionFailed(mint_account.to_string(), e.to_string()))?,
        )
    } else {
        None
    };
    let current_rule_set =
        if let Some(ProgrammableConfig::V1 { rule_set }) = current_md.programmable_config {
            rule_set
        } else {
            None
        };

    Ok((current_md, token, current_rule_set))
}
