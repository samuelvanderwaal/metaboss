use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use structopt::StructOpt;

use metaboss::decode::decode_metadata;
use metaboss::opt::{Command, Opt};
use metaboss::update_metadata::update_metadata;

fn main() -> Result<()> {
    let options = Opt::from_args();

    let client = RpcClient::new(options.rpc.clone());

    match options.cmd {
        Command::Decode {
            ref mint_accounts,
            ref output,
        } => decode_metadata(&client, mint_accounts, output)?,
        Command::UpdateMetadataUri {
            ref keypair,
            ref account,
            ref new_uri,
            ref output,
        } => update_metadata(&client, keypair, account, new_uri, output)?,
    }

    Ok(())
}
