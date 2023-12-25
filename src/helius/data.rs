use mpl_token_metadata::types::Creator;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct HeliusResponse {
    pub helius_result: Vec<HeliusResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusResult {
    pub result: Vec<HeliusAsset>,
    pub pagination_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeliusAsset {
    pub mint: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ByCreatorResponse {
    id: u32,
    jsonrpc: String,
    pub result: ByCreatorResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ByCreatorResult {
    pub total: u32,
    pub limit: u32,
    pub page: u32,
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub interface: String,
    pub id: String,
    pub content: Value,
    pub authorities: Vec<Value>,
    pub compression: Value,
    pub grouping: Value,
    pub royalty: Value,
    pub creators: Vec<Creator>,
    pub ownership: Value,
    pub supply: Value,
    pub mutable: bool,
    pub burnt: bool,
    pub inscription: Option<Inscription>,
    pub spl20: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inscription {
    pub order: u32,
    pub size: u32,
    pub content_type: String,
    pub encoding: String,
    pub validation_hash: String,
    pub inscription_data_account: String,
    pub authority: String,
}
