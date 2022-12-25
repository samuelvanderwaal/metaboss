use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HeliusResponse {
    pub helius_result: Vec<HeliusResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusResult {
    pub result: Vec<Asset>,
    pub pagination_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub mint: String,
    pub name: String,
}
