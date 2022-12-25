use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

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
pub struct UpdateNFTData {
    pub mint_account: String,
    pub nft_data: NFTData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUriData {
    pub mint_account: String,
    pub new_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

#[derive(Debug, Clone)]
pub enum Indexers {
    Helius,
    TheIndexIO,
}

impl FromStr for Indexers {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "helius" => Ok(Indexers::Helius),
            "the_index_io" => Ok(Indexers::TheIndexIO),
            _ => Err(format!("Invalid method: {}", s)),
        }
    }
}

impl Display for Indexers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Indexers::Helius => write!(f, "helius"),
            Indexers::TheIndexIO => write!(f, "the_index_io"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FoundError {
    pub domain: String,
    pub message: String,
}
