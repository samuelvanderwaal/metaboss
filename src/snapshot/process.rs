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
            group_key,
            group_value,
            output,
        } => {
            snapshot_holders(HoldersArgs {
                rpc_url,
                group_key,
                group_value,
                output,
            })
            .await
        }
        SnapshotSubcommands::Mints {
            group_key,
            group_value,
            creator_position,
            output,
        } => {
            snapshot_mints(MintsArgs {
                rpc_url,
                group_key,
                group_value,
                creator_position,
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
        SnapshotSubcommands::Mcc { mcc_id, output } => {
            mcc_mints(MccArgs {
                rpc_url,
                mcc_id,
                output,
            })
            .await
        }
        SnapshotSubcommands::MintsGpa {
            creator,
            position,
            update_authority,
            v2,
            v3,
            allow_unverified,
            output,
        } => snapshot_mints_gpa(
            client,
            SnapshotMintsGpaArgs {
                creator,
                position,
                update_authority,
                v2,
                v3,
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::HoldersGpa {
            update_authority,
            creator,
            position,
            mint_accounts_file,
            v2,
            v3,
            allow_unverified,
            output,
        } => snapshot_holders_gpa(
            client,
            SnapshotHoldersGpaArgs {
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
    }
}
