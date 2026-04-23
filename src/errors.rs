use solana_client::client_error::ClientErrorKind;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("Client Error: '{0}'")]
    ClientError(Box<ClientErrorKind>),

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
    #[error("Migration failed for mint {0}: {1}")]
    MigrationFailed(MintAddress, NetworkError),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Action failed for mint {0}: {1}")]
    UpdateFailed(MintAddress, NetworkError),
}

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Action failed for mint {0}: {1}")]
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

#[cfg(test)]
mod tests {
    use super::*;

    // DecodeError tests

    #[test]
    fn decode_error_network_error_display() {
        let err = DecodeError::NetworkError("timeout".to_string());
        assert_eq!(err.to_string(), "Network Error: 'timeout'");
    }

    #[test]
    fn decode_error_pubkey_parse_failed_display() {
        let err = DecodeError::PubkeyParseFailed("invalid base58".to_string());
        assert_eq!(err.to_string(), "Pubkey Parsing Failed: 'invalid base58'");
    }

    #[test]
    fn decode_error_decode_metadata_failed_display() {
        let err = DecodeError::DecodeMetadataFailed("borsh error".to_string());
        assert_eq!(err.to_string(), "Metadata Decode Failed: 'borsh error'");
    }

    #[test]
    fn decode_error_client_error_display() {
        let kind = ClientErrorKind::Custom("rpc failure".to_string());
        let err = DecodeError::ClientError(Box::new(kind));
        let msg = err.to_string();
        assert!(
            msg.starts_with("Client Error: '"),
            "unexpected display: {msg}"
        );
    }

    // MigrateError tests

    #[test]
    fn migrate_error_display() {
        let err = MigrateError::MigrationFailed(
            "AbcDef123".to_string(),
            "connection refused".to_string(),
        );
        assert_eq!(
            err.to_string(),
            "Migration failed for mint AbcDef123: connection refused"
        );
    }

    // UpdateError tests

    #[test]
    fn update_error_display() {
        let err = UpdateError::UpdateFailed("mint123".to_string(), "tx failed".to_string());
        assert_eq!(err.to_string(), "Action failed for mint mint123: tx failed");
    }

    // ActionError tests

    #[test]
    fn action_error_display() {
        let err = ActionError::ActionFailed("mint456".to_string(), "timeout".to_string());
        assert_eq!(err.to_string(), "Action failed for mint mint456: timeout");
    }

    // SolConfigError tests

    #[test]
    fn sol_config_error_missing_home_display() {
        let err = SolConfigError::MissingHomeEnvVar;
        assert_eq!(err.to_string(), "no home env var found");
    }

    #[test]
    fn sol_config_error_io_error_display() {
        let err = SolConfigError::IOError(io::Error::new(io::ErrorKind::NotFound, "not found"));
        assert_eq!(err.to_string(), "failed to find or open Solana config file");
    }

    #[test]
    fn sol_config_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let err: SolConfigError = io_err.into();
        assert!(matches!(err, SolConfigError::IOError(_)));
    }

    #[test]
    fn sol_config_error_yml_error_display() {
        // Construct an invalid YAML to get a serde_yaml::Error
        let yml_err = serde_yaml::from_str::<serde_yaml::Value>("{{invalid").unwrap_err();
        let err = SolConfigError::YmlError(yml_err);
        assert_eq!(err.to_string(), "failed to deserialize Solana config file");
    }

    #[test]
    fn sol_config_error_from_yml_error() {
        let yml_err = serde_yaml::from_str::<serde_yaml::Value>("{{invalid").unwrap_err();
        let err: SolConfigError = yml_err.into();
        assert!(matches!(err, SolConfigError::YmlError(_)));
    }
}
