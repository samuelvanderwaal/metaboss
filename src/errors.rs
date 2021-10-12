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
