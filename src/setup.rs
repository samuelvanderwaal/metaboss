use anyhow::{anyhow, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    clock::Slot,
    commitment_config::CommitmentConfig,
    hash::Hash,
    signature::{read_keypair_file, Keypair},
};
use std::{fs::File, path::PathBuf, str::FromStr};

pub enum ClientType {
    Standard,
    DAS,
}

#[derive(Debug, Deserialize, Serialize)]
struct SolanaConfig {
    pub json_rpc_url: String,
    pub keypair_path: String,
    pub commitment: String,
}

pub struct CliConfig {
    pub client: RpcClient,
    pub keypair: Keypair,
    pub recent_blockhash: Hash,
    pub recent_slot: Slot,
}

#[derive(Debug, Default)]
pub struct CliConfigBuilder {
    pub json_rpc_url: Option<String>,
    pub keypair_path: Option<PathBuf>,
    pub commitment: Option<String>,
    pub client_type: Option<ClientType>,
}

impl CliConfigBuilder {
    pub fn new() -> Self {
        Self {
            json_rpc_url: None,
            keypair_path: None,
            commitment: None,
            client_type: None,
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
    pub fn client_type(mut self, client_type: Option<ClientType>) -> Self {
        self.client_type = client_type;
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

        let client = RpcClient::new_with_commitment(rpc_url, commitment);

        let keypair_path = self
            .keypair_path
            .clone()
            .ok_or_else(|| anyhow!("No keypair path provided"))?;

        let keypair =
            read_keypair_file(keypair_path).map_err(|_| anyhow!("Unable to read keypair file"))?;

        let recent_blockhash = client.get_latest_blockhash()?;
        let recent_slot = client.get_slot()?;

        Ok(CliConfig {
            client,
            keypair,
            recent_blockhash,
            recent_slot,
        })
    }
}

impl CliConfig {
    pub fn new(
        keypair_path: Option<PathBuf>,
        rpc_url: Option<String>,
        client_type: ClientType,
    ) -> Result<Self> {
        let mut builder = CliConfigBuilder::new();
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

        builder.client_type(Some(client_type));

        let config = builder.build()?;

        Ok(config)
    }

    #[allow(unused)]
    pub fn update_blocks(&mut self) -> Result<()> {
        self.recent_blockhash = self.client.get_latest_blockhash()?;
        self.recent_slot = self.client.get_slot()?;

        Ok(())
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
