use anyhow::Result;
use solana_client::rpc_client::RpcClient;

use crate::burn::burn_one;
use crate::collections::{
    approve_delegate, revoke_delegate, set_and_verify_nft_collection, unverify_nft_collection,
    verify_nft_collection,
};
use crate::decode::{decode_master_edition, decode_metadata};
use crate::derive::{get_cmv2_pda, get_edition_pda, get_generic_pda, get_metadata_pda};
use crate::find::find_missing_editions_process;
use crate::mint::{mint_editions, mint_list, mint_missing_editions, mint_one};
use crate::opt::*;
use crate::sign::{sign_all, sign_one};
use crate::snapshot::{snapshot_cm_accounts, snapshot_holders, snapshot_mints};
use crate::update_metadata::*;
use crate::uses::{approve_use_delegate, revoke_use_delegate, utilize_nft};
use crate::withdraw::{withdraw, WithdrawArgs};

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

pub fn process_collections(client: &RpcClient, commands: CollectionsSubcommands) -> Result<()> {
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
    }
}

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
        ),
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
        } => mint_list(
            client,
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
            set_primary_sale_happened(client, keypair, &account)
        }
        SetSubcommands::UpdateAuthority {
            keypair,
            account,
            new_update_authority,
        } => set_update_authority(client, keypair, &account, &new_update_authority),
        SetSubcommands::UpdateAuthorityAll {
            keypair,
            mint_accounts_file,
            new_update_authority,
        } => set_update_authority_all(client, keypair, &mint_accounts_file, &new_update_authority),
        SetSubcommands::Immutable { keypair, account } => set_immutable(client, keypair, &account),
        SetSubcommands::ImmutableAll {
            keypair,
            mint_accounts_file,
        } => set_immutable_all(client, keypair, &mint_accounts_file),
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

pub fn process_snapshot(client: &RpcClient, commands: SnapshotSubcommands) -> Result<()> {
    match commands {
        SnapshotSubcommands::Holders {
            update_authority,
            creator,
            position,
            mint_accounts_file,
            v2,
            output,
        } => snapshot_holders(
            client,
            &update_authority,
            &creator,
            position,
            &mint_accounts_file,
            v2,
            &output,
        ),
        SnapshotSubcommands::CMAccounts {
            update_authority,
            output,
        } => snapshot_cm_accounts(client, &update_authority, &output),
        SnapshotSubcommands::Mints {
            creator,
            position,
            update_authority,
            v2,
            output,
        } => snapshot_mints(client, &creator, position, update_authority, v2, output),
    }
}

pub fn process_update(client: &RpcClient, commands: UpdateSubcommands) -> Result<()> {
    match commands {
        UpdateSubcommands::Name {
            keypair,
            account,
            new_name,
        } => update_name_one(client, keypair, &account, &new_name),
        UpdateSubcommands::Symbol {
            keypair,
            account,
            new_symbol,
        } => update_symbol_one(client, keypair, &account, &new_symbol),
        UpdateSubcommands::Creators {
            keypair,
            account,
            new_creators,
            append,
        } => update_creator_by_position(client, keypair, &account, &new_creators, append),
        UpdateSubcommands::Data {
            keypair,
            account,
            new_data_file,
        } => update_data_one(client, keypair, &account, &new_data_file),
        UpdateSubcommands::DataAll { keypair, data_dir } => {
            update_data_all(client, keypair, &data_dir)
        }
        UpdateSubcommands::Uri {
            keypair,
            account,
            new_uri,
        } => update_uri_one(client, keypair, &account, &new_uri),
        UpdateSubcommands::UriAll { keypair, json_file } => {
            update_uri_all(client, keypair, &json_file)
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
