#[macro_use]
extern crate log;

use anyhow::Result;
use metaboss::check::process_check;
use metaboss::constants::PUBLIC_RPC_URLS;
use metaboss::extend_program::process_extend_program;
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

use metaboss::constants::*;
use metaboss::opt::*;
use metaboss::parse::parse_solana_config;
use metaboss::process_subcommands::*;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Opt::from_args();

    let log_level = format!("solana={}", options.log_level);
    solana_logger::setup_with_default(&log_level);

    let sol_config = parse_solana_config();

    let (rpc, commitment) = if let Some(cli_rpc) = options.rpc {
        (cli_rpc, String::from("confirmed"))
    } else if let Some(config) = sol_config {
        (config.json_rpc_url, config.commitment)
    } else {
        info!(
            "Could not find a valid Solana-CLI config file. Defaulting to https://devnet.genesysgo.net devnet node."
        );
        (
            String::from("https://devnet.genesysgo.net"),
            String::from("confirmed"),
        )
    };

    // Set rate limiting if the user specified a public RPC.
    if PUBLIC_RPC_URLS.contains(&rpc.as_str()) {
        warn!(
            "Using a public RPC URL is not recommended for heavy tasks as you will be rate-limited and suffer a performance hit"
        );
        warn!("Please use a private RPC endpoint for best performance results.");
        *USE_RATE_LIMIT.write().unwrap() = true;
    } else if RATE_LIMIT_DELAYS.contains_key(&rpc.as_str()) {
        *USE_RATE_LIMIT.write().unwrap() = true;
        *RPC_DELAY_NS.write().unwrap() = RATE_LIMIT_DELAYS[&rpc.as_str()];
    }

    let commitment = CommitmentConfig::from_str(&commitment)?;
    let timeout = Duration::from_secs(options.timeout);

    let client = RpcClient::new_with_timeout_and_commitment(rpc.clone(), timeout, commitment);
    let async_client =
        AsyncRpcClient::new_with_timeout_and_commitment(rpc.clone(), timeout, commitment);

    match options.cmd {
        Command::Collections {
            collections_subcommands,
        } => process_collections(client, async_client, collections_subcommands).await?,
        Command::Airdrop {
            airdrop_subcommands,
        } => process_airdrop(client, airdrop_subcommands).await?,
        Command::Burn { burn_subcommands } => process_burn_asset(client, burn_subcommands).await?,
        Command::BurnNft {
            burn_nft_subcommands,
        } => process_burn_nft(client, burn_nft_subcommands).await?,
        Command::BurnPrint {
            burn_print_subcommands,
        } => process_burn_print(client, burn_print_subcommands).await?,
        Command::Check { check_subcommands } => process_check(check_subcommands).await?,
        Command::Create { create_subcommands } => process_create(client, create_subcommands)?,
        Command::Decode { decode_subcommands } => process_decode(&client, decode_subcommands)?,
        Command::Derive { derive_subcommands } => process_derive(derive_subcommands),
        Command::ExtendProgram {
            keypair_path,
            program_address,
            additional_bytes,
        } => process_extend_program(client, keypair_path, program_address, additional_bytes)?,
        Command::Find { find_subcommands } => process_find(&client, find_subcommands)?,
        Command::Mint { mint_subcommands } => process_mint(&client, mint_subcommands)?,
        Command::ParseErrors {
            parse_errors_file_subcommands,
        } => process_parse_errors_file(parse_errors_file_subcommands)?,
        Command::Set { set_subcommands } => process_set(client, set_subcommands).await?,
        Command::Sign { sign_subcommands } => process_sign(&client, sign_subcommands)?,
        Command::Snapshot {
            snapshot_subcommands,
        } => process_snapshot(client, snapshot_subcommands).await?,
        Command::Transfer {
            transfer_subcommands,
        } => process_transfer(client, transfer_subcommands)?,
        Command::Update { update_subcommands } => {
            process_update(client, update_subcommands).await?
        }
        Command::Uses { uses_subcommands } => process_uses(&client, uses_subcommands)?,
        Command::Verify { verify_subcommands } => {
            process_verify(client, verify_subcommands).await?
        }
        Command::Unverify {
            unverify_subcommands,
        } => process_unverify(client, unverify_subcommands).await?,
    }

    Ok(())
}
