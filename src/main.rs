#[macro_use]
extern crate log;

use anyhow::Result;
use metaboss::airdrop::process_airdrop;
use metaboss::check::process_check;
use metaboss::extend_program::process_extend_program;
use metaboss::setup::AppConfigBuilder;
use structopt::StructOpt;

use metaboss::opt::*;
use metaboss::process_subcommands::*;
use metaboss::snapshot::process_snapshot;

#[tokio::main]
async fn main() -> Result<()> {
    let options = Opt::from_args();

    let log_level = format!("solana={}", options.log_level);
    solana_logger::setup_with_default(&log_level);

    let mut builder = AppConfigBuilder::new().timeout(options.timeout);

    if let Some(rpc) = options.rpc {
        builder = builder.rpc_url(rpc);
    }

    let config = builder.build()?;
    let rpc = config.rpc_url;
    let client = config.client;
    let async_client = config.async_client;

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
        Command::Mint { mint_subcommands } => process_mint(client, mint_subcommands)?,
        Command::ParseErrors {
            parse_errors_file_subcommands,
        } => process_parse_errors_file(parse_errors_file_subcommands)?,
        Command::Set { set_subcommands } => process_set(client, set_subcommands).await?,
        Command::Sign { sign_subcommands } => process_sign(&client, sign_subcommands)?,
        Command::Snapshot {
            snapshot_subcommands,
        } => process_snapshot(client, rpc, snapshot_subcommands).await?,
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
