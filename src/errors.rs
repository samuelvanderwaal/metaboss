use solana_client::client_error::ClientErrorKind;
use std::io;
use thiserror::Error;

pub type MintAddress = String;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("no account data found")]
    MissingAccount(MintAddress),

    #[error("failed to get account data")]
    ClientError(MintAddress, ClientErrorKind),

    #[error("network request failed with error: {1}")]
    NetworkError(MintAddress, String),

    #[error("failed to parse string into Pubkey")]
    PubkeyParseFailed(MintAddress),

    #[error("failed to decode metadata")]
    DecodeMetadataFailed(MintAddress, String),
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
