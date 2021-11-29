use anyhow::Result;
use metaboss::constants::PUBLIC_RPC_URLS;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

use metaboss::constants::*;
use metaboss::opt::*;
use metaboss::parse::parse_solana_config;
use metaboss::process_subcommands::*;

fn main() -> Result<()> {
    let sol_config = parse_solana_config();

    let (mut rpc, commitment) = if let Some(config) = sol_config {
        (config.json_rpc_url, config.commitment)
    } else {
        eprintln!(
            "Could not find a valid Solana-CLI config file. Please specify a RPC manually with '-r' or set up your Solana-CLI config file."
        );
        std::process::exit(1);
    };

    let options = Opt::from_args();

    if let Some(cli_rpc) = options.rpc {
        rpc = cli_rpc.clone();
    }

    // Set rate limiting if the user specified a public RPC.
    if PUBLIC_RPC_URLS.contains(&rpc.as_str()) {
        eprintln!(
            r#"
            WARNING: Using a public RPC URL is not recommended for heavy tasks as you will be rate-limited and suffer a performance hit.
            Please use a private RPC endpoint for best performance results."#
        );
        *USE_RATE_LIMIT.write().unwrap() = true;
    }

    let commitment = CommitmentConfig::from_str(&commitment)?;
    let timeout = Duration::from_secs(options.timeout);

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
