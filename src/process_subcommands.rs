use anyhow::Result;
use solana_client::rpc_client::RpcClient;

use crate::decode::decode_metadata;
use crate::mint::{mint_list, mint_one};
use crate::opt::*;
use crate::sign::{sign_all, sign_one};
use crate::snapshot::{snapshot_cm_accounts, snapshot_holders, snapshot_mints};
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
            immutable,
        } => mint_one(&client, &keypair, &receiver, nft_data_file, immutable),
        MintSubcommands::List {
            keypair,
            receiver,
            nft_data_dir,
            immutable,
        } => mint_list(&client, keypair, receiver, nft_data_dir, immutable),
    }
}

pub fn process_update(client: &RpcClient, commands: UpdateSubcommands) -> Result<()> {
    match commands {
        UpdateSubcommands::Data {
            keypair,
            account,
            new_data_file,
        } => update_data(&client, &keypair, &account, &new_data_file),
        UpdateSubcommands::Uri {
            keypair,
            account,
            new_uri,
        } => update_uri(&client, &keypair, &account, &new_uri),
    }
}

pub fn process_set(client: &RpcClient, commands: SetSubcommands) -> Result<()> {
    match commands {
        SetSubcommands::PrimarySaleHappened { keypair, account } => {
            set_primary_sale_happened(&client, &keypair, &account)
        }
        SetSubcommands::UpdateAuthority {
            keypair,
            account,
            new_update_authority,
        } => {
            set_update_authority(&client, &keypair, &account, &new_update_authority)
        }
        SetSubcommands::UpdateAuthorityAll {
            keypair,
            accounts,
            new_update_authority
        } => {
            set_update_authority_all(&client, &keypair, &accounts, &new_update_authority)
        }
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
        SnapshotSubcommands::CMAccounts {
            update_authority,
            output,
        } => snapshot_cm_accounts(&client, &update_authority, &output),
        SnapshotSubcommands::Mints {
            candy_machine_id,
            update_authority,
            output,
        } => snapshot_mints(&client, candy_machine_id, update_authority, output),
    }
}
