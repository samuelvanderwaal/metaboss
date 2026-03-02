use anyhow::{anyhow, Result};
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair},
};

use std::{path::PathBuf, str::FromStr, time::Duration};

use crate::constants::{PUBLIC_RPC_URLS, RATE_LIMIT_DELAYS, RPC_DELAY_NS, USE_RATE_LIMIT};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum ClientType {
    Standard,
    DAS,
}

pub enum ClientLike {
    RpcClient(RpcClient),
    DasClient(Client),
}

pub struct CliConfig {
    pub client: ClientLike,
    pub keypair: Option<Keypair>,
    pub rpc_url: String,
}

#[derive(Debug)]
pub struct CliConfigBuilder {
    pub json_rpc_url: Option<String>,
    pub keypair_path: Option<PathBuf>,
    pub commitment: Option<String>,
    pub client_type: ClientType,
}

impl CliConfigBuilder {
    pub fn new(client_type: ClientType) -> Self {
        Self {
            json_rpc_url: None,
            keypair_path: None,
            commitment: None,
            client_type,
        }
    }
    pub fn rpc_url(mut self, json_rpc_url: String) -> Self {
        self.json_rpc_url = Some(json_rpc_url);
        self
    }
    pub fn keypair_path(mut self, keypair_path: PathBuf) -> Self {
        self.keypair_path = Some(keypair_path);
        self
    }
    pub fn commitment(mut self, commitment: String) -> Self {
        self.commitment = Some(commitment);
        self
    }

    pub fn build(&self) -> Result<CliConfig> {
        let rpc_url = self
            .json_rpc_url
            .clone()
            .ok_or_else(|| anyhow!("No rpc url provided"))?;

        let commitment = match self.commitment.clone() {
            Some(commitment) => CommitmentConfig::from_str(&commitment)?,
            None => CommitmentConfig::confirmed(),
        };

        let client = match self.client_type {
            ClientType::Standard => {
                ClientLike::RpcClient(RpcClient::new_with_commitment(rpc_url.clone(), commitment))
            }
            ClientType::DAS => ClientLike::DasClient(Client::new()),
        };

        let keypair = if let Some(keypair_path) = &self.keypair_path {
            let keypair = read_keypair_file(keypair_path)
                .map_err(|_| anyhow!("Unable to read keypair file"))?;

            Some(keypair)
        } else {
            None
        };

        Ok(CliConfig {
            client,
            keypair,
            rpc_url,
        })
    }
}

impl CliConfig {
    pub fn new(
        keypair_path: Option<PathBuf>,
        rpc_url: Option<String>,
        client_type: ClientType,
    ) -> Result<Self> {
        let mut builder = CliConfigBuilder::new(client_type);
        let solana_config = crate::parse::parse_solana_config();

        if let Some(config) = solana_config {
            builder = builder
                .rpc_url(config.json_rpc_url)
                .keypair_path(config.keypair_path.into())
                .commitment(config.commitment);
        }

        if let Some(keypair_path) = keypair_path {
            builder = builder.keypair_path(keypair_path);
        }

        if let Some(rpc_url) = rpc_url {
            builder = builder.rpc_url(rpc_url);
        }

        let config = builder.build()?;

        Ok(config)
    }
}

const DEFAULT_TIMEOUT_SECS: u64 = 90;
const DEFAULT_RPC_URL: &str = "https://devnet.genesysgo.net";

/// Configuration for the main application, including RPC clients and rate limiting.
pub struct AppConfig {
    pub client: RpcClient,
    pub async_client: AsyncRpcClient,
    pub rpc_url: String,
}

/// Builder for constructing the main application configuration.
///
/// Resolves RPC endpoint, commitment level, and timeout from CLI arguments,
/// the Solana CLI config file, or built-in defaults. Automatically configures
/// rate limiting for known public RPC endpoints.
pub struct AppConfigBuilder {
    rpc_url: Option<String>,
    timeout_secs: u64,
}

impl AppConfigBuilder {
    pub fn new() -> Self {
        Self {
            rpc_url: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    /// Set the RPC endpoint URL, overriding the Solana config file value.
    pub fn rpc_url(mut self, rpc_url: String) -> Self {
        self.rpc_url = Some(rpc_url);
        self
    }

    /// Set the RPC client timeout in seconds. Defaults to 90 seconds.
    pub fn timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Build the `AppConfig`, resolving values from the Solana CLI config
    /// file as needed and configuring rate limiting for public RPC endpoints.
    pub fn build(self) -> Result<AppConfig> {
        let sol_config = crate::parse::parse_solana_config();

        let (rpc_url, commitment_str) = if let Some(cli_rpc) = self.rpc_url {
            (cli_rpc, String::from("confirmed"))
        } else if let Some(config) = sol_config {
            (config.json_rpc_url, config.commitment)
        } else {
            info!(
                "Could not find a valid Solana-CLI config file. Defaulting to {} devnet node.",
                DEFAULT_RPC_URL
            );
            (String::from(DEFAULT_RPC_URL), String::from("confirmed"))
        };

        // Configure rate limiting for known public RPC endpoints.
        if PUBLIC_RPC_URLS.contains(&rpc_url.as_str()) {
            warn!(
                "Using a public RPC URL is not recommended for heavy tasks as you will be rate-limited and suffer a performance hit"
            );
            warn!("Please use a private RPC endpoint for best performance results.");
            *USE_RATE_LIMIT.write().unwrap() = true;
        } else if RATE_LIMIT_DELAYS.contains_key(&rpc_url.as_str()) {
            *USE_RATE_LIMIT.write().unwrap() = true;
            *RPC_DELAY_NS.write().unwrap() = RATE_LIMIT_DELAYS[&rpc_url.as_str()];
        }

        let commitment = CommitmentConfig::from_str(&commitment_str)?;
        let timeout = Duration::from_secs(self.timeout_secs);

        let client =
            RpcClient::new_with_timeout_and_commitment(rpc_url.clone(), timeout, commitment);
        let async_client =
            AsyncRpcClient::new_with_timeout_and_commitment(rpc_url.clone(), timeout, commitment);

        Ok(AppConfig {
            client,
            async_client,
            rpc_url,
        })
    }
}

impl Default for AppConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CliConfigBuilder tests ---

    #[test]
    fn cli_config_builder_new_has_correct_defaults() {
        let builder = CliConfigBuilder::new(ClientType::Standard);
        assert!(builder.json_rpc_url.is_none());
        assert!(builder.keypair_path.is_none());
        assert!(builder.commitment.is_none());
        assert_eq!(builder.client_type, ClientType::Standard);
    }

    #[test]
    fn cli_config_builder_new_das_type() {
        let builder = CliConfigBuilder::new(ClientType::DAS);
        assert_eq!(builder.client_type, ClientType::DAS);
    }

    #[test]
    fn cli_config_builder_sets_rpc_url() {
        let builder =
            CliConfigBuilder::new(ClientType::Standard).rpc_url("https://example.com".to_string());
        assert_eq!(
            builder.json_rpc_url,
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn cli_config_builder_sets_keypair_path() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .keypair_path(PathBuf::from("/tmp/keypair.json"));
        assert_eq!(
            builder.keypair_path,
            Some(PathBuf::from("/tmp/keypair.json"))
        );
    }

    #[test]
    fn cli_config_builder_sets_commitment() {
        let builder =
            CliConfigBuilder::new(ClientType::Standard).commitment("finalized".to_string());
        assert_eq!(builder.commitment, Some("finalized".to_string()));
    }

    #[test]
    fn cli_config_builder_chained_methods() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://example.com".to_string())
            .keypair_path(PathBuf::from("/tmp/key.json"))
            .commitment("confirmed".to_string());
        assert_eq!(
            builder.json_rpc_url,
            Some("https://example.com".to_string())
        );
        assert_eq!(builder.keypair_path, Some(PathBuf::from("/tmp/key.json")));
        assert_eq!(builder.commitment, Some("confirmed".to_string()));
    }

    #[test]
    fn cli_config_builder_build_fails_without_rpc_url() {
        let builder = CliConfigBuilder::new(ClientType::Standard);
        let result = builder.build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No rpc url provided"));
    }

    #[test]
    fn cli_config_builder_build_succeeds_with_rpc_url() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://api.devnet.solana.com".to_string());
        let config = builder.build();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.rpc_url, "https://api.devnet.solana.com");
        assert!(config.keypair.is_none());
    }

    #[test]
    fn cli_config_builder_build_with_invalid_commitment_fails() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://api.devnet.solana.com".to_string())
            .commitment("not_a_valid_commitment".to_string());
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn cli_config_builder_build_defaults_to_confirmed_commitment() {
        // When no commitment is set, build should succeed (defaults to confirmed).
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://api.devnet.solana.com".to_string());
        let config = builder.build();
        assert!(config.is_ok());
    }

    #[test]
    fn cli_config_builder_build_with_invalid_keypair_fails() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://api.devnet.solana.com".to_string())
            .keypair_path(PathBuf::from("/nonexistent/keypair.json"));
        let result = builder.build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unable to read keypair file"));
    }

    #[test]
    fn cli_config_builder_build_creates_standard_client() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://api.devnet.solana.com".to_string());
        let config = builder.build().unwrap();
        assert!(matches!(config.client, ClientLike::RpcClient(_)));
    }

    #[test]
    fn cli_config_builder_build_creates_das_client() {
        let builder = CliConfigBuilder::new(ClientType::DAS)
            .rpc_url("https://api.devnet.solana.com".to_string());
        let config = builder.build().unwrap();
        assert!(matches!(config.client, ClientLike::DasClient(_)));
    }

    #[test]
    fn cli_config_builder_last_rpc_url_wins() {
        let builder = CliConfigBuilder::new(ClientType::Standard)
            .rpc_url("https://first.com".to_string())
            .rpc_url("https://second.com".to_string());
        let config = builder.build().unwrap();
        assert_eq!(config.rpc_url, "https://second.com");
    }

    // --- AppConfigBuilder tests ---

    #[test]
    fn app_config_builder_new_has_correct_defaults() {
        let builder = AppConfigBuilder::new();
        assert!(builder.rpc_url.is_none());
        assert_eq!(builder.timeout_secs, DEFAULT_TIMEOUT_SECS);
    }

    #[test]
    fn app_config_builder_default_matches_new() {
        let from_new = AppConfigBuilder::new();
        let from_default = AppConfigBuilder::default();
        assert_eq!(from_new.rpc_url, from_default.rpc_url);
        assert_eq!(from_new.timeout_secs, from_default.timeout_secs);
    }

    #[test]
    fn app_config_builder_sets_rpc_url() {
        let builder = AppConfigBuilder::new().rpc_url("https://custom-rpc.example.com".to_string());
        assert_eq!(
            builder.rpc_url,
            Some("https://custom-rpc.example.com".to_string())
        );
    }

    #[test]
    fn app_config_builder_sets_timeout() {
        let builder = AppConfigBuilder::new().timeout(120);
        assert_eq!(builder.timeout_secs, 120);
    }

    #[test]
    fn app_config_builder_chained_methods() {
        let builder = AppConfigBuilder::new()
            .rpc_url("https://custom-rpc.example.com".to_string())
            .timeout(30);
        assert_eq!(
            builder.rpc_url,
            Some("https://custom-rpc.example.com".to_string())
        );
        assert_eq!(builder.timeout_secs, 30);
    }

    #[test]
    fn app_config_builder_build_with_explicit_rpc_url() {
        let config = AppConfigBuilder::new()
            .rpc_url("https://custom-rpc.example.com".to_string())
            .build()
            .unwrap();
        assert_eq!(config.rpc_url, "https://custom-rpc.example.com");
    }

    #[test]
    fn app_config_builder_build_without_rpc_falls_back() {
        // Without an explicit RPC URL and without a Solana config file,
        // the builder should fall back to the default RPC URL.
        let config = AppConfigBuilder::new().build().unwrap();
        // It should either use the Solana config file value or the default.
        // In a test environment without a Solana config, it falls back to DEFAULT_RPC_URL.
        assert!(!config.rpc_url.is_empty());
    }

    #[test]
    fn app_config_builder_build_with_custom_timeout() {
        // The timeout is applied to the RPC clients internally.
        // We verify the builder accepts the value and the build succeeds.
        let config = AppConfigBuilder::new()
            .rpc_url("https://api.devnet.solana.com".to_string())
            .timeout(30)
            .build()
            .unwrap();
        assert_eq!(config.rpc_url, "https://api.devnet.solana.com");

        // Clean up.
        *USE_RATE_LIMIT.write().unwrap() = false;
    }

    #[test]
    fn app_config_builder_public_rpc_enables_rate_limiting() {
        // Reset rate limit state before test.
        *USE_RATE_LIMIT.write().unwrap() = false;

        let _config = AppConfigBuilder::new()
            .rpc_url("https://api.devnet.solana.com".to_string())
            .build()
            .unwrap();

        assert!(*USE_RATE_LIMIT.read().unwrap());

        // Clean up.
        *USE_RATE_LIMIT.write().unwrap() = false;
    }

    #[test]
    fn app_config_builder_private_rpc_does_not_enable_rate_limiting() {
        // Reset rate limit state before test.
        *USE_RATE_LIMIT.write().unwrap() = false;

        let _config = AppConfigBuilder::new()
            .rpc_url("https://my-private-rpc.example.com".to_string())
            .build()
            .unwrap();

        assert!(!*USE_RATE_LIMIT.read().unwrap());
    }

    #[test]
    fn app_config_builder_last_rpc_url_wins() {
        let config = AppConfigBuilder::new()
            .rpc_url("https://first.com".to_string())
            .rpc_url("https://second.com".to_string())
            .build()
            .unwrap();
        assert_eq!(config.rpc_url, "https://second.com");
    }

    #[test]
    fn default_timeout_is_90_seconds() {
        assert_eq!(DEFAULT_TIMEOUT_SECS, 90);
    }

    #[test]
    fn default_rpc_url_is_devnet() {
        assert_eq!(DEFAULT_RPC_URL, "https://devnet.genesysgo.net");
    }
}
