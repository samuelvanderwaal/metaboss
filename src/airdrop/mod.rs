pub mod sol;
pub mod spl;
pub use sol::*;
pub use spl::*;

pub use std::{collections::HashMap, fs::File, path::PathBuf, str::FromStr};

pub use anyhow::Result;
pub use jib::{Jib, Network};
pub use log::debug;
pub use serde::{Deserialize, Serialize};
pub use solana_client::rpc_client::RpcClient;
pub use solana_sdk::{pubkey::Pubkey, signer::Signer};

pub use crate::update::{parse_keypair, parse_solana_config};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct FailedTransaction {
    transaction_accounts: Vec<String>,
    recipients: HashMap<String, u64>,
    error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Recipient {
    address: String,
    amount: u64,
}

pub const PRIORITY_FEE: u64 = 25_000;
