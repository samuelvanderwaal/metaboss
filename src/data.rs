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

#[cfg(test)]
mod tests {
    use super::*;

    // FromStr tests

    #[test]
    fn from_str_helius() {
        let indexer: Indexers = "helius".parse().unwrap();
        assert!(matches!(indexer, Indexers::Helius));
    }

    #[test]
    fn from_str_the_index_io() {
        let indexer: Indexers = "the_index_io".parse().unwrap();
        assert!(matches!(indexer, Indexers::TheIndexIO));
    }

    #[test]
    fn from_str_invalid_string() {
        let result = Indexers::from_str("not_a_real_indexer");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid method: not_a_real_indexer");
    }

    #[test]
    fn from_str_empty_string() {
        let result = Indexers::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn from_str_is_case_sensitive() {
        assert!(Indexers::from_str("Helius").is_err());
        assert!(Indexers::from_str("HELIUS").is_err());
        assert!(Indexers::from_str("The_Index_IO").is_err());
        assert!(Indexers::from_str("THE_INDEX_IO").is_err());
    }

    #[test]
    fn from_str_rejects_leading_trailing_whitespace() {
        assert!(Indexers::from_str(" helius").is_err());
        assert!(Indexers::from_str("helius ").is_err());
        assert!(Indexers::from_str(" the_index_io ").is_err());
    }

    // Display tests

    #[test]
    fn display_helius() {
        assert_eq!(Indexers::Helius.to_string(), "helius");
    }

    #[test]
    fn display_the_index_io() {
        assert_eq!(Indexers::TheIndexIO.to_string(), "the_index_io");
    }

    // Round-trip tests

    #[test]
    fn round_trip_helius() {
        let original = Indexers::Helius;
        let parsed: Indexers = original.to_string().parse().unwrap();
        assert_eq!(parsed.to_string(), original.to_string());
    }

    #[test]
    fn round_trip_the_index_io() {
        let original = Indexers::TheIndexIO;
        let parsed: Indexers = original.to_string().parse().unwrap();
        assert_eq!(parsed.to_string(), original.to_string());
    }

    // FoundError tests

    #[test]
    fn found_error_stores_fields() {
        let err = FoundError {
            domain: "solana".to_string(),
            message: "something went wrong".to_string(),
        };
        assert_eq!(err.domain, "solana");
        assert_eq!(err.message, "something went wrong");
    }

    #[test]
    fn found_error_debug_output() {
        let err = FoundError {
            domain: "test".to_string(),
            message: "msg".to_string(),
        };
        let debug = format!("{:?}", err);
        assert!(debug.contains("test"));
        assert!(debug.contains("msg"));
    }

    #[test]
    fn found_error_serialization_round_trip() {
        let err = FoundError {
            domain: "metaplex".to_string(),
            message: "decode failed".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let deserialized: FoundError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.domain, "metaplex");
        assert_eq!(deserialized.message, "decode failed");
    }
}
