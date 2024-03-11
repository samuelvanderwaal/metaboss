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
    data::Priority,
    decode::{decode_metadata_from_mint, ToPubkey},
    derive::{derive_edition_pda, derive_metadata_pda},
    nft::get_nft_token_account,
    update::{update_asset, UpdateAssetArgs, V1UpdateArgs},
};
pub use mpl_token_metadata::{
    accounts::Metadata,
    instructions::{SetTokenStandardBuilder, UpdateMetadataAccountV2Builder},
    types::Data,
    types::{DataV2, ProgrammableConfig, RuleSetToggle, TokenStandard, UpdateArgs},
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
use std::fs::File;
pub use std::{cmp, fmt::Display, str::FromStr, sync::Arc};

pub use crate::cache::{Action, BatchActionArgs, Cache, NewValue, RunActionArgs};
pub use crate::decode::{decode, get_metadata_pda};
pub use crate::errors::ActionError;
pub use crate::parse::parse_solana_config;
pub use crate::parse::{parse_cli_creators, parse_keypair};

pub fn parse_mint_list(
    mint_list_file: Option<String>,
    cache_file: &Option<String>,
) -> AnyResult<Option<Vec<String>>> {
    if cache_file.is_none() {
        let mint_file = mint_list_file
            .ok_or_else(|| anyhow!("Must provide either a mint list or a cache file!"))?;
        let f = File::open(mint_file)?;
        let mint_list = serde_json::from_reader(f)?;
        Ok(Some(mint_list))
    } else {
        Ok(None)
    }
}
