use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

use metaboss::opt::*;
use metaboss::parse::parse_solana_config;
use metaboss::process_subcommands::*;

fn main() -> Result<()> {
    let sol_config = parse_solana_config();

    let (mut rpc, commitment) = if let Some(config) = sol_config {
        (config.json_rpc_url, config.commitment)
    } else {
        (
            "https://api.devnet.solana.com".to_string(),
            "confirmed".to_string(),
        )
    };

    let options = Opt::from_args();

    if let Some(cli_rpc) = options.rpc {
        rpc = cli_rpc.clone();
    }
    let commitment = CommitmentConfig::from_str(&commitment)?;

    let timeout = Duration::from_secs(60);

    let client = RpcClient::new_with_timeout_and_commitment(rpc, timeout, commitment);
    match options.cmd {
        Command::Decode { decode_subcommands } => process_decode(&client, decode_subcommands)?,
        Command::Mint { mint_subcommands } => process_mint(&client, mint_subcommands)?,
        Command::Update { update_subcommands } => process_update(&client, update_subcommands)?,
        Command::Set { set_subcommands } => process_set(&client, set_subcommands)?,
        Command::Sign { sign_subcommands } => process_sign(&client, sign_subcommands)?,
        Command::Snapshot {
            snapshot_subcommands,
        } => process_snapshot(&client, snapshot_subcommands)?,
    }

    Ok(())
}
