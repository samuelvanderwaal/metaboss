use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use structopt::StructOpt;

use metaboss::decode::decode_metadata;
use metaboss::opt::{Command, Opt};
use metaboss::update_metadata::{update_metadata_uri, update_metadata_uri_all};

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
            ref mint_account,
            ref new_uri,
        } => update_metadata_uri(&client, keypair, mint_account, new_uri)?,
        Command::UpdateMetadataUriAll {
            ref keypair,
            ref mint_accounts,
        } => update_metadata_uri_all(&client, keypair, mint_accounts)?,
    }

    Ok(())
}
