use anyhow::Result;
use metaboss::decode::{decode_metadata, decode_metadata_all};
use metaboss::opt::{Command, Opt};
use metaboss::sign::sign;
use metaboss::snapshot::{get_mints, get_snapshot};
use metaboss::update_metadata::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use structopt::StructOpt;

use metaboss::parse::parse_solana_config;

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

    let client = RpcClient::new_with_commitment(rpc, commitment);
    match options.cmd {
        Command::Decode {
            ref mint_account,
            ref output,
        } => decode_metadata(&client, mint_account, output)?,
        Command::DecodeAll {
            ref json_file,
            ref output,
        } => decode_metadata_all(&client, json_file, output)?,
        Command::GetMints {
            ref update_authority,
            ref candy_machine_id,
            ref output,
        } => get_mints(&client, update_authority, candy_machine_id, output)?,
        Command::UpdateNFT {
            ref keypair,
            ref mint_account,
            ref new_uri,
        } => update_nft(&client, keypair, mint_account, new_uri)?,
        Command::UpdateNFTAll {
            ref keypair,
            ref json_file,
        } => update_nft_all(&client, keypair, json_file)?,
        Command::SetUpdateAuthority {
            ref keypair,
            ref mint_account,
            ref new_update_authority,
        } => set_update_authority(&client, keypair, mint_account, new_update_authority)?,
        Command::SetUpdateAuthorityAll {
            ref keypair,
            ref json_file,
        } => set_update_authority_all(&client, keypair, json_file)?,
        Command::Sign {
            ref keypair,
            ref candy_machine_id,
            ref mint,
        } => sign(&client, keypair, candy_machine_id, mint)?,
        Command::Snapshot {
            ref update_authority,
            ref candy_machine_id,
            ref output,
        } => get_snapshot(&client, update_authority, candy_machine_id, output)?,
    }

    Ok(())
}
