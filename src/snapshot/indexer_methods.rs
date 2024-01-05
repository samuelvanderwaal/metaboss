use std::fmt::Display;

use crate::data::Indexers;

#[derive(Debug, Clone)]
pub struct NftsByCreatorArgs {
    pub creator: String,
    pub api_key: String,
    pub indexer: Indexers,
    pub output: String,
}

#[derive(Debug, Clone)]
pub struct NftsByCollectionArgs {
    pub collection: String,
    pub api_key: String,
    pub indexer: Indexers,
    pub output: String,
}

#[derive(Debug, Clone)]
pub enum Method {
    Creator,
    Collection,
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Creator => write!(f, "creator"),
            Method::Collection => write!(f, "collection"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GetMintsArgs {
    pub address: String,
    pub method: Method,
    pub api_key: String,
    pub indexer: Indexers,
    pub output: String,
}
