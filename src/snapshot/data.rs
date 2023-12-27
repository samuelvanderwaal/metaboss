use super::*;

pub const PARALLEL_LIMIT: usize = 50;
pub type HolderResults = Vec<Result<Holder>>;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Holder {
    pub owner_wallet: String,
    pub mint_account: String,
    pub metadata_account: String,
    pub associated_token_address: String,
}

#[derive(Debug, Serialize)]
pub struct CandyMachineProgramAccounts {
    pub config_accounts: Vec<ConfigAccount>,
    pub candy_machine_accounts: Vec<CandyMachineAccount>,
}

#[derive(Debug, Serialize)]
pub struct ConfigAccount {
    pub address: String,
    pub data_len: usize,
}

#[derive(Debug, Serialize)]
pub struct CandyMachineAccount {
    pub address: String,
    pub data_len: usize,
}

pub struct SnapshotMintsGpaArgs {
    pub creator: Option<String>,
    pub position: usize,
    pub update_authority: Option<String>,
    pub v2: bool,
    pub v3: bool,
    pub allow_unverified: bool,
    pub output: String,
}

pub struct SnapshotHoldersGpaArgs {
    pub creator: Option<String>,
    pub position: usize,
    pub update_authority: Option<String>,
    pub mint_accounts_file: Option<String>,
    pub v2: bool,
    pub v3: bool,
    pub allow_unverified: bool,
    pub output: String,
}

use mpl_token_metadata::types::Creator;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DasResponse {
    id: u32,
    jsonrpc: String,
    pub result: DasResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DasResult {
    pub total: u32,
    pub limit: u32,
    pub page: u32,
    pub items: Vec<Item>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ByCreatorResult {
    pub total: u32,
    pub limit: u32,
    pub page: u32,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub interface: String,
    pub id: String,
    pub content: Value,
    pub authorities: Vec<Value>,
    pub compression: Value,
    pub grouping: Value,
    pub royalty: Value,
    pub creators: Vec<Creator>,
    pub ownership: Ownership,
    pub supply: Value,
    pub mutable: bool,
    pub burnt: bool,
    pub inscription: Option<Inscription>,
    pub spl20: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Inscription {
    pub order: u32,
    pub size: u32,
    pub content_type: String,
    pub encoding: String,
    pub validation_hash: String,
    pub inscription_data_account: String,
    pub authority: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Ownership {
    pub delegate: Option<String>,
    pub delegated: bool,
    pub frozen: bool,
    pub owner: String,
    pub ownership_model: String,
}
