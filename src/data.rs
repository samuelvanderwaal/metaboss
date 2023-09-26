use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

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
            _ => Err(format!("Invalid method: {s}")),
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
