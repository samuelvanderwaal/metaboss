use anyhow::Result as AnyResult;
use async_trait::async_trait;
use mpl_token_metadata::{
    accounts::{Edition, Metadata},
    instructions::{BurnEditionNft, BurnNft},
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};

use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::{str::FromStr, sync::Arc};

use crate::{
    cache::{Action, BatchActionArgs, RunActionArgs},
    derive::{derive_edition_marker_pda, derive_edition_pda, derive_metadata_pda},
    errors::ActionError,
    parse::{parse_keypair, parse_solana_config},
    utils::get_largest_token_account_owner,
};

mod burn_legacy;
pub use burn_legacy::*;
mod burn_asset;
pub use burn_asset::*;
