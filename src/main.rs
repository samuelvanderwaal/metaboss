use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use structopt::StructOpt;

use metaboss::decode::decode_metadata_all;
use metaboss::opt::{Command, Opt};
use metaboss::update_metadata::*;

fn main() -> Result<()> {
    let options = Opt::from_args();

    let client = RpcClient::new(options.rpc.clone());

    match options.cmd {
        Command::Decode {
            ref mint_accounts,
            ref output,
        } => decode_metadata_all(&client, mint_accounts, output)?,
        Command::SetUri {
            ref keypair,
            ref mint_account,
            ref new_uri,
        } => set_uri(&client, keypair, mint_account, new_uri)?,
        Command::SetUriAll {
            ref keypair,
            ref json_file,
        } => set_uri_all(&client, keypair, json_file)?,
        Command::SetUpdateAuthority {
            ref keypair,
            ref mint_account,
            ref new_update_authority,
        } => set_update_authority(&client, keypair, mint_account, new_update_authority)?,
        Command::SetUpdateAuthorityAll {
            ref keypair,
            ref json_file,
        } => set_update_authority_all(&client, keypair, json_file)?,
    }

    Ok(())
}
