use anyhow::Result;
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};

use crate::burn::{burn_all, burn_one, BurnAllArgs};
use crate::collections::{
    approve_delegate, check_collection_items, get_collection_items, migrate_collection,
    revoke_delegate, set_and_verify_nft_collection, set_size, unverify_nft_collection,
    verify_nft_collection, MigrateArgs,
};
use crate::create::{create_fungible, create_metadata, CreateFungibleArgs, CreateMetadataArgs};
use crate::decode::{decode_master_edition, decode_metadata};
use crate::derive::{get_cmv2_pda, get_edition_pda, get_generic_pda, get_metadata_pda};
use crate::find::find_missing_editions_process;
use crate::mint::{mint_editions, mint_list, mint_missing_editions, mint_one};
use crate::opt::*;
use crate::parse::{parse_errors_code, parse_errors_file};
use crate::sign::{sign_all, sign_one};
use crate::snapshot::{
    snapshot_cm_accounts, snapshot_holders, snapshot_indexed_holders, snapshot_indexed_mints,
    snapshot_mints, SnapshotHoldersArgs, SnapshotMintsArgs,
};
use crate::update::*;
use crate::uses::{approve_use_delegate, revoke_use_delegate, utilize_nft};

pub fn process_uses(client: &RpcClient, commands: UsesSubcommands) -> Result<()> {
    match commands {
        UsesSubcommands::ApproveAuthority {
            keypair,
            delegate_use_authority,
            mint_nft,
            token_account_nft,
            burner_program_id,
            number_of_uses,
        } => approve_use_delegate(
            client,
            keypair,
            mint_nft,
            delegate_use_authority,
            token_account_nft,
            burner_program_id,
            number_of_uses,
        ),

        UsesSubcommands::RevokeAuthority {
            delegate_use_authority,
            keypair,
            mint_nft,
            token_account_nft,
        } => revoke_use_delegate(
            client,
            keypair,
            mint_nft,
            delegate_use_authority,
            token_account_nft,
        ),

        UsesSubcommands::Utilize {
            burner_program_id,
            is_delegate,
            keypair,
            mint_nft,
            holder_nft,
            token_account_nft,
        } => utilize_nft(
            client,
            keypair,
            mint_nft,
            holder_nft,
            token_account_nft,
            burner_program_id,
            is_delegate,
        ),
    }
}

pub async fn process_collections(
    client: RpcClient,
    async_client: AsyncRpcClient,
    commands: CollectionsSubcommands,
) -> Result<()> {
    match commands {
        CollectionsSubcommands::VerifyCollection {
            keypair,
            collection_mint,
            nft_mint,
            is_delegate,
        } => verify_nft_collection(client, keypair, nft_mint, collection_mint, is_delegate),

        CollectionsSubcommands::UnverifyCollection {
            keypair,
            collection_mint,
            is_delegate,
            nft_mint,
        } => unverify_nft_collection(client, keypair, nft_mint, collection_mint, is_delegate),

        CollectionsSubcommands::SetAndVerifyCollection {
            keypair,
            nft_mint,
            update_authority_nft,
            collection_mint,
            is_delegate,
        } => set_and_verify_nft_collection(
            client,
            keypair,
            nft_mint,
            collection_mint,
            update_authority_nft,
            is_delegate,
        ),

        CollectionsSubcommands::ApproveAuthority {
            keypair,
            collection_mint,
            delegate_authority,
        } => approve_delegate(client, keypair, collection_mint, delegate_authority),

        CollectionsSubcommands::RevokeAuthority {
            keypair,
            collection_mint,
            delegate_authority,
        } => revoke_delegate(client, keypair, collection_mint, delegate_authority),

        CollectionsSubcommands::SetSize {
            keypair,
            collection_mint,
            size,
        } => set_size(client, keypair, collection_mint, size),

        CollectionsSubcommands::Migrate {
            keypair,
            mint_address,
            candy_machine_id,
            mint_list,
            cache_file,
            retries,
            batch_size,
            output_file,
        } => {
            migrate_collection(MigrateArgs {
                client,
                async_client,
                keypair,
                mint_address,
                candy_machine_id,
                mint_list,
                cache_file,
                retries,
                batch_size,
                output_file,
            })
            .await
        }
        CollectionsSubcommands::GetItems {
            collection_mint,
            method,
            api_key,
        } => get_collection_items(collection_mint, method, api_key).await,

        CollectionsSubcommands::CheckItems {
            collection_mint,
            item_list,
            debug,
        } => check_collection_items(async_client, collection_mint, item_list, debug).await,
    }
}

pub async fn process_burn(client: RpcClient, commands: BurnSubcommands) -> Result<()> {
    match commands {
        BurnSubcommands::One { keypair, account } => burn_one(client, keypair, account).await,
        BurnSubcommands::All {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            burn_all(BurnAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
    }
}

pub fn process_create(client: RpcClient, commands: CreateSubcommands) -> Result<()> {
    match commands {
        CreateSubcommands::Metadata {
            keypair,
            mint,
            metadata,
            immutable,
        } => create_metadata(CreateMetadataArgs {
            client,
            keypair,
            mint,
            metadata,
            immutable,
        }),
        CreateSubcommands::Fungible {
            keypair,
            metadata,
            decimals,
            initial_supply,
            immutable,
        } => create_fungible(CreateFungibleArgs {
            client,
            keypair,
            metadata,
            decimals,
            initial_supply,
            immutable,
        }),
    }
}

pub fn process_decode(client: &RpcClient, commands: DecodeSubcommands) -> Result<()> {
    match commands {
        DecodeSubcommands::Mint {
            account,
            full,
            list_file,
            raw,
            ref output,
        } => decode_metadata(
            client,
            account.as_ref(),
            full,
            list_file.as_ref(),
            raw,
            output,
        )?,
        DecodeSubcommands::Master { account } => decode_master_edition(client, &account)?,
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

pub fn process_find(client: &RpcClient, commands: FindSubcommands) -> Result<()> {
    match commands {
        FindSubcommands::MissingEditions { account } => {
            find_missing_editions_process(client, &account)
        }
        FindSubcommands::Error { error_code } => parse_errors_code(&error_code),
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
            max_editions,
            sign,
            sized,
        } => mint_one(
            client,
            keypair,
            &receiver,
            nft_data_file,
            external_metadata_uri.as_ref(),
            immutable,
            primary_sale_happened,
            max_editions,
            sign,
            sized,
        )
        .map(|_| ()),
        MintSubcommands::Editions {
            keypair,
            account,
            receiver,
            next_editions,
            specific_editions,
        } => mint_editions(
            client,
            keypair,
            account,
            &receiver,
            next_editions,
            specific_editions,
        ),
        MintSubcommands::MissingEditions { keypair, account } => {
            mint_missing_editions(client, &keypair, &account)
        }
        MintSubcommands::List {
            keypair,
            receiver,
            nft_data_dir,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
            track,
        } => mint_list(
            client,
            keypair,
            receiver,
            nft_data_dir,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
            track,
        ),
    }
}

pub async fn process_set(client: RpcClient, commands: SetSubcommands) -> Result<()> {
    match commands {
        SetSubcommands::PrimarySaleHappened { keypair, account } => {
            set_primary_sale_happened_one(client, keypair, &account)
        }
        SetSubcommands::PrimarySaleHappenedAll {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            set_primary_sale_happened_all(SetPrimarySaleHappenedAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
        SetSubcommands::UpdateAuthority {
            keypair,
            account,
            new_update_authority,
            keypair_payer,
        } => set_update_authority_one(
            &client,
            keypair,
            &account,
            &new_update_authority,
            keypair_payer,
        ),
        SetSubcommands::UpdateAuthorityAll {
            keypair,
            payer,
            mint_list,
            new_authority,
            cache_file,
            batch_size,
            retries,
        } => {
            set_update_authority_all(SetUpdateAuthorityAllArgs {
                client,
                keypair,
                payer,
                mint_list,
                new_authority,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
        SetSubcommands::Immutable { keypair, account } => {
            set_immutable_one(&client, keypair, &account)
        }
        SetSubcommands::ImmutableAll {
            keypair,
            mint_list,
            cache_file,
            batch_size,
            retries,
        } => {
            set_immutable_all(SetImmutableAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                batch_size,
                retries,
            })
            .await
        }
    }
}

pub fn process_sign(client: &RpcClient, commands: SignSubcommands) -> Result<()> {
    match commands {
        SignSubcommands::One { keypair, account } => sign_one(client, keypair, account),
        SignSubcommands::All {
            keypair,
            creator,
            position,
            v2,
            mint_accounts_file,
        } => sign_all(client, keypair, &creator, position, v2, mint_accounts_file),
    }
}

pub async fn process_snapshot(client: &RpcClient, commands: SnapshotSubcommands) -> Result<()> {
    match commands {
        SnapshotSubcommands::Holders {
            update_authority,
            creator,
            position,
            mint_accounts_file,
            v2,
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
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::IndexedHolders {
            indexer,
            api_key,
            creator,
            output,
        } => snapshot_indexed_holders(indexer, api_key, &creator, &output).await,
        SnapshotSubcommands::CMAccounts {
            update_authority,
            output,
        } => snapshot_cm_accounts(client, &update_authority, &output),
        SnapshotSubcommands::Mints {
            creator,
            position,
            update_authority,
            v2,
            allow_unverified,
            output,
        } => snapshot_mints(
            client,
            SnapshotMintsArgs {
                creator,
                position,
                update_authority,
                v2,
                allow_unverified,
                output,
            },
        ),
        SnapshotSubcommands::IndexedMints {
            indexer,
            api_key,
            creator,
            output,
        } => snapshot_indexed_mints(indexer, api_key, &creator, output).await,
    }
}

pub async fn process_update(client: RpcClient, commands: UpdateSubcommands) -> Result<()> {
    match commands {
        UpdateSubcommands::SellerFeeBasisPoints {
            keypair,
            account,
            new_seller_fee_basis_points,
        } => update_seller_fee_basis_points_one(
            &client,
            keypair,
            &account,
            &new_seller_fee_basis_points,
        ),
        UpdateSubcommands::SellerFeeBasisPointsAll {
            keypair,
            mint_list,
            cache_file,
            new_sfbp,
            batch_size,
            retries,
        } => {
            update_seller_fee_basis_points_all(UpdateSellerFeeBasisPointsAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                new_sfbp,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::Name {
            keypair,
            account,
            new_name,
        } => update_name_one(&client, keypair, &account, &new_name),
        UpdateSubcommands::Symbol {
            keypair,
            account,
            new_symbol,
        } => update_symbol_one(client, keypair, account, new_symbol).await,
        UpdateSubcommands::SymbolAll {
            keypair,
            mint_list,
            cache_file,
            new_symbol,
            batch_size,
            retries,
        } => {
            update_symbol_all(UpdateSymbolAllArgs {
                client,
                keypair,
                mint_list,
                cache_file,
                new_symbol,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::Creators {
            keypair,
            account,
            new_creators,
            append,
        } => update_creator_by_position(&client, keypair, &account, &new_creators, append).await,
        UpdateSubcommands::CreatorsAll {
            keypair,
            mint_list,
            cache_file,
            new_creators,
            append,
            batch_size,
            retries,
        } => {
            update_creator_all(UpdateCreatorAllArgs {
                client,
                keypair_path: keypair,
                mint_list,
                cache_file,
                new_creators,
                should_append: append,
                batch_size,
                retries,
            })
            .await
        }
        UpdateSubcommands::Data {
            keypair,
            account,
            new_data_file,
        } => update_data_one(&client, keypair, &account, &new_data_file),
        UpdateSubcommands::DataAll { keypair, data_dir } => {
            update_data_all(&client, keypair, &data_dir)
        }
        UpdateSubcommands::Uri {
            keypair,
            account,
            new_uri,
        } => update_uri_one(&client, keypair, &account, &new_uri),
        UpdateSubcommands::UriAll { keypair, json_file } => {
            update_uri_all(&client, keypair, &json_file)
        }
    }
}

pub fn process_parse_errors_file(commands: ParseErrorsSubCommands) -> Result<()> {
    match commands {
        ParseErrorsSubCommands::File => parse_errors_file(),
    }
}
