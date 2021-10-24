use anyhow::Result;
use solana_client::rpc_client::RpcClient;

use crate::decode::decode_metadata;
use crate::mint::{mint_list, mint_one};
use crate::opt::*;
use crate::sign::{sign_all, sign_one};
use crate::snapshot::{get_cm_accounts, get_mints, snapshot_holders};
use crate::update_metadata::*;

pub fn process_decode(client: &RpcClient, commands: DecodeSubcommands) -> Result<()> {
    match commands {
        DecodeSubcommands::Mint {
            account,
            list_file,
            ref output,
        } => decode_metadata(client, account.as_ref(), list_file.as_ref(), output)?,
    }
    Ok(())
}

pub fn process_mint(client: &RpcClient, commands: MintSubcommands) -> Result<()> {
    match commands {
        MintSubcommands::One {
            keypair,
            receiver,
            nft_data_file,
        } => mint_one(&client, &keypair, &receiver, nft_data_file),
        MintSubcommands::List {
            keypair,
            receiver,
            nft_data_dir,
        } => mint_list(&client, keypair, receiver, nft_data_dir),
    }
}

pub fn process_sign(client: &RpcClient, commands: SignSubcommands) -> Result<()> {
    match commands {
        SignSubcommands::One { keypair, account } => sign_one(&client, keypair, account),
        SignSubcommands::All {
            keypair,
            candy_machine_id,
            mint_accounts_file,
        } => sign_all(&client, &keypair, candy_machine_id, mint_accounts_file),
    }
}

pub fn process_snapshot(client: &RpcClient, commands: SnapshotSubcommands) -> Result<()> {
    match commands {
        SnapshotSubcommands::Holders {
            update_authority,
            candy_machine_id,
            output,
        } => snapshot_holders(&client, &update_authority, &candy_machine_id, &output),
    }
}
