use std::path::PathBuf;

use anyhow::Result;
use metaboss_lib::check::{check_metadata_value, MetadataValue};
use mpl_token_metadata::state::Metadata;
use solana_client::rpc_client::RpcClient;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum CheckSubcommands {
    /// Burn a batch of assets.
    #[structopt(name = "metadata-value")]
    MetadataValue {
        /// Path to the directory of metadata files
        #[structopt(short = "d", long)]
        metadata_files_dir: PathBuf,

        /// Metadata value
        value: MetadataValue,
    },
}

pub async fn process_check(commands: CheckSubcommands) -> Result<()> {
    match commands {
        CheckSubcommands::MetadataValue {
            metadata_files_dir,
            value,
        } => check_value_all(metadata_files_dir, value),
    }
}

pub struct CheckValueAllArgs {
    pub client: RpcClient,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
    pub value: MetadataValue,
}

fn check_value_all(metadata_file_dir: PathBuf, value: MetadataValue) -> Result<()> {
    let metadata_files = std::fs::read_dir(metadata_file_dir)?;

    let mut failed_mints = Vec::new();

    for file in metadata_files {
        let file = file?;
        let path = file.path();

        let metadata = std::fs::read_to_string(&path)?;

        let metadata: Metadata = serde_json::from_str(&metadata)?;

        if !check_metadata_value(metadata, &value) {
            failed_mints.push(path);
        }
    }

    if !failed_mints.is_empty() {
        println!("Files with metadata that don't match the specified value:");
        for mint in failed_mints {
            println!("{}", mint.display());
        }
    } else {
        println!("All metadata files have the specified value!");
    }

    Ok(())
}
