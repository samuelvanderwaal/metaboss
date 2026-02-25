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
