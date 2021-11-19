use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUri {
    mint_account: String,
    new_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<NFTCreator>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}
