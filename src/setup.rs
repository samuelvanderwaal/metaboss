use anyhow::{anyhow, Result};
use dirs::home_dir;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{read_keypair_file, Keypair},
};

use std::{fs::File, path::PathBuf, str::FromStr};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum ClientType {
    Standard,
    DAS,
}

pub enum ClientLike {
    RpcClient(RpcClient),
    DasClient(Client),
}

#[derive(Debug, Deserialize, Serialize)]
struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
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

// impl Default for ClientType {
//     fn default() -> Self {
//         Self::Standard
//     }
// }

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
        let solana_config = parse_solana_config();

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

fn parse_solana_config() -> Option<SolanaConfig> {
    let home_path = home_dir().expect("Couldn't find home dir");

    let solana_config_path = home_path
        .join(".config")
        .join("solana")
        .join("cli")
        .join("config.yml");

    let config_file = File::open(solana_config_path).ok();

    if let Some(config_file) = config_file {
        let config: SolanaConfig = serde_yaml::from_reader(config_file).ok()?;
        return Some(config);
    }
    None
}
