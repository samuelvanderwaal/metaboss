use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("no account data found")]
    MissingAccount(String),

    #[error("failed to parse string into Pubkey")]
    PubkeyParseFailed,

    #[error("failed to decode metadata")]
    DecodeMetadataFailed(String),
}

#[derive(Error, Debug)]
pub enum SolConfigError {
    #[error("no home env var found")]
    MissingHomeEnvVar,

    #[error("failed to find or open Solana config file")]
    IOError(#[from] io::Error),

    #[error("failed to deserialize Solana config file")]
    YmlError(#[from] serde_yaml::Error),
}
