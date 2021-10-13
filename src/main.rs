use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use structopt::StructOpt;

use metaboss::decode::decode_metadata;
use metaboss::opt::{Command, Opt};
use metaboss::snapshot::{get_mints, get_snapshot};
use metaboss::update_metadata::*;

fn main() -> Result<()> {
    let options = Opt::from_args();

    let client = RpcClient::new(options.rpc.clone());

    match options.cmd {
        Command::Decode {
            ref json_file,
            ref output,
        } => decode_metadata(&client, json_file, output)?,
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
        Command::Snapshot {
            ref update_authority,
            ref candy_machine_id,
            ref output,
        } => get_snapshot(&client, update_authority, candy_machine_id, output)?,
    }

    Ok(())
}
