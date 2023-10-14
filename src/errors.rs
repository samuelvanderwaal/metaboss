use solana_client::client_error::ClientErrorKind;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Client Error: '{0}'")]
    ClientError(ClientErrorKind),

    #[error("Network Error: '{0}'")]
    NetworkError(String),

    #[error("Pubkey Parsing Failed: '{0}'")]
    PubkeyParseFailed(String),

    #[error("Metadata Decode Failed: '{0}'")]
    DecodeMetadataFailed(String),
}

pub type MintAddress = String;
pub type NetworkError = String;

#[derive(Error, Debug)]
pub enum MigrateError {
    #[error("Migration failed with error: {1}")]
    MigrationFailed(MintAddress, NetworkError),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Action failed with error: {1}")]
    UpdateFailed(MintAddress, NetworkError),
}

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Action failed with error: {1}")]
    ActionFailed(MintAddress, NetworkError),
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
