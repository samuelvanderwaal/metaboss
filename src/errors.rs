use solana_client::client_error::ClientErrorKind;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("no account data found")]
    MissingAccount(String),

    #[error("failed to get account data")]
    ClientError(ClientErrorKind),

    #[error("network request failed after three attempts: ensure you used a valid address and check the state of the Solana cluster")]
    NetworkError(String),

    #[error("failed to parse string into Pubkey")]
    PubkeyParseFailed(String),

    #[error("failed to decode metadata")]
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
