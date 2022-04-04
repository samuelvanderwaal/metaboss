#[macro_use]
extern crate log;

use anyhow::Result;
use env_logger::{Builder, Target};
use log::LevelFilter;
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

fn setup_logging(log_level: String) -> Result<()> {
    let level = LevelFilter::from_str(log_level.as_str())?;
    Builder::new()
        .filter_level(level)
        .target(Target::Stdout)
        .init();
    Ok(())
}

fn main() -> Result<()> {
    let options = Opt::from_args();

    setup_logging(options.log_level)?;

    let sol_config = parse_solana_config();

    let (rpc, commitment) = if let Some(cli_rpc) = options.rpc {
        (cli_rpc, String::from("confirmed"))
    } else if let Some(config) = sol_config {
        (config.json_rpc_url, config.commitment)
    } else {
        info!(
            "Could not find a valid Solana-CLI config file. Defaulting to https://psytrbhymqlkfrhudd.dev.genesysgo.net:8899/ devnet node."
        );
        (
            String::from("https://psytrbhymqlkfrhudd.dev.genesysgo.net:8899/"),
            String::from("confirmed"),
        )
    };

    // Set rate limiting if the user specified a public RPC.
    if PUBLIC_RPC_URLS.contains(&rpc.as_str()) {
        warn!(
            "Using a public RPC URL is not recommended for heavy tasks as you will be rate-limited and suffer a performance hit.
        Please use a private RPC endpoint for best performance results."
        );
        *USE_RATE_LIMIT.write().unwrap() = true;
    } else if RATE_LIMIT_DELAYS.contains_key(&rpc.as_str()) {
        *USE_RATE_LIMIT.write().unwrap() = true;
        *RPC_DELAY_NS.write().unwrap() = RATE_LIMIT_DELAYS[&rpc.as_str()];
    }

    let commitment = CommitmentConfig::from_str(&commitment)?;
    let timeout = Duration::from_secs(options.timeout);

    let client = RpcClient::new_with_timeout_and_commitment(rpc.clone(), timeout, commitment);
    match options.cmd {
        Command::Collections {
            collections_subcommands,
        } => process_collections(&client, collections_subcommands)?,
        Command::Uses { uses_subcommands } => process_uses(&client, uses_subcommands)?,
        Command::Burn { burn_subcommands } => process_burn(&client, burn_subcommands)?,
        Command::Decode { decode_subcommands } => process_decode(&client, decode_subcommands)?,
        Command::Derive { derive_subcommands } => process_derive(derive_subcommands),
        Command::Find { find_subcommands } => process_find(&client, find_subcommands)?,
        Command::Mint { mint_subcommands } => process_mint(&client, mint_subcommands)?,
        Command::Update { update_subcommands } => process_update(&client, update_subcommands)?,
        Command::Set { set_subcommands } => process_set(&client, set_subcommands)?,
        Command::Sign { sign_subcommands } => process_sign(&client, sign_subcommands)?,
        Command::Snapshot {
            snapshot_subcommands,
        } => process_snapshot(&client, snapshot_subcommands)?,
        Command::Withdraw {
            withdraw_subcommands,
        } => process_withdraw(rpc, withdraw_subcommands)?,
    }

    println!("Done!");
    Ok(())
}
