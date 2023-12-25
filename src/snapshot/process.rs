use anyhow::Result;

use solana_client::rpc_client::RpcClient;

use crate::opt::SnapshotSubcommands;

use super::*;

pub async fn process_snapshot(
    client: RpcClient,
    rpc_url: String,
    commands: SnapshotSubcommands,
) -> Result<()> {
    match commands {
        SnapshotSubcommands::Holders {
            update_authority,
            creator,
            position,
            mint_accounts_file,
            v2,
            v3,
            allow_unverified,
            output,
        } => snapshot_holders(
            client,
            SnapshotHoldersArgs {
                update_authority,
                creator,
                position,
                mint_accounts_file,
                v2,
                v3,
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::IndexedHolders {
            indexer,
            api_key,
            creator,
            output,
        } => {
            snapshot_indexed_holders(NftsByCreatorArgs {
                creator,
                api_key,
                indexer,
                output,
            })
            .await
        }
        SnapshotSubcommands::CMAccounts {
            update_authority,
            output,
        } => snapshot_cm_accounts(client, &update_authority, &output),
        SnapshotSubcommands::Mints {
            creator,
            position,
            update_authority,
            v2,
            v3,
            allow_unverified,
            output,
        } => snapshot_mints(
            client,
            SnapshotMintsArgs {
                creator,
                position,
                update_authority,
                v2,
                v3,
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::IndexedMints {
            indexer,
            api_key,
            creator,
            output,
        } => {
            snapshot_indexed_mints(NftsByCreatorArgs {
                creator,
                api_key,
                indexer,
                output,
            })
            .await
        }
        SnapshotSubcommands::MintsByCreator {
            indexer,
            api_key,
            address,
            output,
        } => {
            snapshot_mints_by_creator(GetMintsArgs {
                indexer,
                api_key,
                method: Method::Creator,
                address,
                output,
            })
            .await
        }
        SnapshotSubcommands::MintsByCollection {
            indexer,
            api_key,
            address,
            output,
        } => {
            snapshot_mints_by_collection(GetMintsArgs {
                indexer,
                api_key,
                method: Method::Collection,
                address,
                output,
            })
            .await
        }
        SnapshotSubcommands::Prints {
            master_mint,
            creator,
            output,
        } => {
            snapshot_print_editions(SnapshotPrintEditionsArgs {
                client,
                master_mint,
                creator,
                output,
            })
            .await
        }
        SnapshotSubcommands::Fvca { creator, output } => {
            fcva_mints(FcvaArgs {
                rpc_url,
                creator,
                output,
            })
            .await
        }
    }
}
