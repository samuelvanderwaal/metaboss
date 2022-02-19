use anyhow::Result;
use solana_client::rpc_client::RpcClient;

use crate::burn::burn_one;
use crate::decode::decode_metadata;
use crate::derive::{get_cmv2_pda, get_edition_pda, get_generic_pda, get_metadata_pda};
use crate::mint::{mint_list, mint_one};
use crate::opt::*;
use crate::sign::{sign_all, sign_one};
use crate::snapshot::{snapshot_cm_accounts, snapshot_holders, snapshot_mints};
use crate::update_metadata::*;
use crate::withdraw::{withdraw, WithdrawArgs};

pub fn process_burn(client: &RpcClient, commands: BurnSubcommands) -> Result<()> {
    match commands {
        BurnSubcommands::One { keypair, account } => burn_one(client, keypair, account),
    }
}

pub fn process_decode(client: &RpcClient, commands: DecodeSubcommands) -> Result<()> {
    match commands {
        DecodeSubcommands::Mint {
            account,
            full,
            list_file,
            ref output,
        } => decode_metadata(client, account.as_ref(), full, list_file.as_ref(), output)?,
    }
    Ok(())
}

pub fn process_derive(commands: DeriveSubcommands) {
    match commands {
        DeriveSubcommands::Pda { seeds, program_id } => get_generic_pda(seeds, program_id),
        DeriveSubcommands::Metadata { mint_account } => get_metadata_pda(mint_account),
        DeriveSubcommands::Edition { mint_account } => get_edition_pda(mint_account),
        DeriveSubcommands::CMV2Creator { candy_machine_id } => get_cmv2_pda(candy_machine_id),
    }
}

pub fn process_mint(client: &RpcClient, commands: MintSubcommands) -> Result<()> {
    match commands {
        MintSubcommands::One {
            keypair,
            receiver,
            nft_data_file,
            external_metadata_uri,
            immutable,
            primary_sale_happened,
            sign,
        } => mint_one(
            &client,
            &keypair,
            &receiver,
            nft_data_file,
            external_metadata_uri.as_ref(),
            immutable,
            primary_sale_happened,
            sign,
        ),
        MintSubcommands::List {
            keypair,
            receiver,
            nft_data_dir,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
        } => mint_list(
            &client,
            keypair,
            receiver,
            nft_data_dir,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
        ),
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
        } => set_update_authority(&client, &keypair, &account, &new_update_authority),
        SetSubcommands::UpdateAuthorityAll {
            keypair,
            mint_accounts_file,
            new_update_authority,
        } => set_update_authority_all(
            &client,
            &keypair,
            &mint_accounts_file,
            &new_update_authority,
        ),
        SetSubcommands::Immutable { keypair, account } => {
            set_immutable(&client, &keypair, &account)
        }
        SetSubcommands::ImmutableAll {
            keypair,
            mint_accounts_file,
        } => set_immutable_all(&client, &keypair, &mint_accounts_file),
    }
}

pub fn process_sign(client: &RpcClient, commands: SignSubcommands) -> Result<()> {
    match commands {
        SignSubcommands::One { keypair, account } => sign_one(&client, keypair, account),
        SignSubcommands::All {
            keypair,
            candy_machine_id,
            v2,
            mint_accounts_file,
        } => sign_all(&client, &keypair, candy_machine_id, v2, mint_accounts_file),
    }
}

pub fn process_snapshot(client: &RpcClient, commands: SnapshotSubcommands) -> Result<()> {
    match commands {
        SnapshotSubcommands::Holders {
            update_authority,
            candy_machine_id,
            v2,
            output,
        } => snapshot_holders(&client, &update_authority, &candy_machine_id, v2, &output),
        SnapshotSubcommands::CMAccounts {
            update_authority,
            output,
        } => snapshot_cm_accounts(&client, &update_authority, &output),
        SnapshotSubcommands::Mints {
            candy_machine_id,
            update_authority,
            v2,
            output,
        } => snapshot_mints(&client, candy_machine_id, update_authority, v2, output),
    }
}

pub fn process_update(client: &RpcClient, commands: UpdateSubcommands) -> Result<()> {
    match commands {
        UpdateSubcommands::Data {
            keypair,
            account,
            new_data_file,
        } => update_data_one(&client, &keypair, &account, &new_data_file),
        UpdateSubcommands::DataAll { keypair, data_dir } => {
            update_data_all(&client, &keypair, &data_dir)
        }
        UpdateSubcommands::Uri {
            keypair,
            account,
            new_uri,
        } => update_uri_one(&client, &keypair, &account, &new_uri),
        UpdateSubcommands::UriAll { keypair, json_file } => {
            update_uri_all(&client, &keypair, &json_file)
        }
    }
}

pub fn process_withdraw(rpc_url: String, commands: WithdrawSubcommands) -> Result<()> {
    match commands {
        WithdrawSubcommands::CMV2 {
            candy_machine_id,
            keypair,
        } => withdraw(WithdrawArgs {
            rpc_url,
            keypair,
            candy_machine_id,
        }),
    }
}
