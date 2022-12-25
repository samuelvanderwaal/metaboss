use std::fmt::Display;

use crate::{data::Indexers, helius, theindexio};
use anyhow::Result;

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

pub async fn snapshot_mints_by_creator(args: GetMintsArgs) -> Result<()> {
    match args.indexer {
        Indexers::Helius => {
            helius::get_mints(args).await?;
        }
        Indexers::TheIndexIO => {
            theindexio::get_mints(args).await?;
        }
    }
    Ok(())
}

pub async fn snapshot_mints_by_collection(args: GetMintsArgs) -> Result<()> {
    match args.indexer {
        Indexers::Helius => helius::get_mints(args).await,
        Indexers::TheIndexIO => theindexio::get_mints(args).await,
    }
}
