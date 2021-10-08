use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use structopt::StructOpt;

use metaboss::decode::decode_metadata;
use metaboss::opt::{Command, Opt};

fn main() -> Result<()> {
    let options = Opt::from_args();

    let client = RpcClient::new(options.rpc.clone());

    match options.cmd {
        Command::Decode {
            ref mint_accounts,
            ref output,
        } => decode_metadata(client, mint_accounts, output)?,
    }

    Ok(())
}
