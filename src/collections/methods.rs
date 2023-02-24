use super::common::*;
use crate::parse::parse_keypair;
use crate::{
    derive::{derive_collection_authority_record, derive_edition_pda, derive_metadata_pda},
    parse::parse_solana_config,
    utils::send_and_confirm_transaction,
};
use mpl_token_metadata::instruction::{
    set_and_verify_sized_collection_item, set_collection_size, unverify_sized_collection_item,
    verify_sized_collection_item,
};
use mpl_token_metadata::state::TokenMetadataAccount;
use solana_sdk::account::ReadableAccount;
use solana_sdk::commitment_config::CommitmentConfig;

pub const OPEN_FILES_LIMIT: usize = 1024;

pub fn set_and_verify_nft_collection(
    client: RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    nft_auth: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_metadata = derive_metadata_pda(&Pubkey::from_str(&nft_mint)?);
    let nft_update_authority = Pubkey::from_str(&nft_auth)?;
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_md_pubkey = derive_metadata_pda(&collection_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(derive_collection_authority_record(&collection_pubkey, &keypair.pubkey()).0),
        false => None,
    };

    // Is it a sized collection?
    let collection_md_account = client.get_account_data(&collection_md_pubkey)?;
    let collection_metadata = Metadata::safe_deserialize(collection_md_account.as_slice())?;

    let set_and_verify_ix = if collection_metadata.collection_details.is_some() {
        set_and_verify_sized_collection_item(
            metadata_program_id(),
            nft_metadata,
            keypair.pubkey(),
            keypair.pubkey(),
            nft_update_authority,
            collection_pubkey,
            collection_md_pubkey,
            collection_edition_pubkey,
            collection_authority_record,
        )
    } else {
        set_and_verify_collection(
            metadata_program_id(),
            nft_metadata,
            keypair.pubkey(),
            keypair.pubkey(),
            nft_update_authority,
            collection_pubkey,
            collection_md_pubkey,
            collection_edition_pubkey,
            collection_authority_record,
        )
    };

    send_and_confirm_transaction(&client, keypair, &[set_and_verify_ix])?;

    Ok(())
}

pub fn unverify_nft_collection(
    client: RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_metadata = derive_metadata_pda(&Pubkey::from_str(&nft_mint)?);
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_md_pubkey = derive_metadata_pda(&collection_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(derive_collection_authority_record(&collection_pubkey, &keypair.pubkey()).0),
        false => None,
    };

    // We need to check if the parent collection NFT exists because people sometimes burn them.
    let collection_md_account_opt = client
        .get_account_with_commitment(&collection_md_pubkey, CommitmentConfig::confirmed())?
        .value;

    let unverify_collection_ix = if let Some(collection_md_account) = collection_md_account_opt {
        let collection_metadata = Metadata::safe_deserialize(collection_md_account.data())?;

        // Choose which handler to use based on if collection is sized or not.
        if collection_metadata.collection_details.is_some() {
            unverify_sized_collection_item(
                metadata_program_id(),
                nft_metadata,
                keypair.pubkey(),
                keypair.pubkey(),
                collection_pubkey,
                collection_md_pubkey,
                collection_edition_pubkey,
                collection_authority_record,
            )
        } else {
            unverify_collection(
                metadata_program_id(),
                nft_metadata,
                keypair.pubkey(),
                collection_pubkey,
                collection_md_pubkey,
                collection_edition_pubkey,
                collection_authority_record,
            )
        }
    } else {
        // Account is not found so presumed burned so we can use either handler.
        unverify_collection(
            metadata_program_id(),
            nft_metadata,
            keypair.pubkey(),
            collection_pubkey,
            collection_md_pubkey,
            collection_edition_pubkey,
            collection_authority_record,
        )
    };

    send_and_confirm_transaction(&client, keypair, &[unverify_collection_ix])?;

    Ok(())
}

pub fn verify_nft_collection(
    client: RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_metadata = derive_metadata_pda(&Pubkey::from_str(&nft_mint)?);
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_md_pubkey = derive_metadata_pda(&collection_pubkey);
    let collection_edition_pubkey = derive_edition_pda(&collection_pubkey);
    let collection_authority_record = match is_delegate_present {
        true => Some(derive_collection_authority_record(&collection_pubkey, &keypair.pubkey()).0),
        false => None,
    };

    // Is it a sized collection?
    let collection_md_account = client.get_account_data(&collection_md_pubkey)?;
    let collection_metadata = Metadata::safe_deserialize(collection_md_account.as_slice())?;

    // Choose which handler to use based on if collection is sized or not.
    let verify_collection_ix = if collection_metadata.collection_details.is_some() {
        verify_sized_collection_item(
            metadata_program_id(),
            nft_metadata,
            keypair.pubkey(),
            keypair.pubkey(),
            collection_pubkey,
            collection_md_pubkey,
            collection_edition_pubkey,
            collection_authority_record,
        )
    } else {
        verify_collection(
            metadata_program_id(),
            nft_metadata,
            keypair.pubkey(),
            keypair.pubkey(),
            collection_pubkey,
            collection_md_pubkey,
            collection_edition_pubkey,
            collection_authority_record,
        )
    };

    send_and_confirm_transaction(&client, keypair, &[verify_collection_ix])?;

    Ok(())
}

pub fn approve_delegate(
    client: RpcClient,
    keypair_path: Option<String>,
    collection_mint: String,
    delegate_authority: String,
) -> AnyResult<()> {
    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let (collection_authority_record, _bump) =
        derive_collection_authority_record(&collection_pubkey, &delegate_pubkey);

    let metadata = derive_metadata_pda(&collection_pubkey);

    let approve_collection_auth_ix = approve_collection_authority(
        metadata_program_id(),
        collection_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        keypair.pubkey(),
        metadata,
        collection_pubkey,
    );

    send_and_confirm_transaction(&client, keypair, &[approve_collection_auth_ix])?;

    Ok(())
}

pub fn revoke_delegate(
    client: RpcClient,
    keypair_path: Option<String>,
    collection_mint: String,
    delegate_authority: String,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let collection_pubkey = Pubkey::from_str(&collection_mint)?;
    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let (collection_authority_record, _bump) =
        derive_collection_authority_record(&collection_pubkey, &delegate_pubkey);

    let metadata = derive_metadata_pda(&collection_pubkey);

    let revoke_collection_auth_ix = revoke_collection_authority(
        metadata_program_id(),
        collection_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        metadata,
        collection_pubkey,
    );

    send_and_confirm_transaction(&client, keypair, &[revoke_collection_auth_ix])?;

    Ok(())
}

pub fn set_size(
    client: RpcClient,
    keypair_path: Option<String>,
    collection_mint: String,
    size: u64,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let collection_mint_pubkey = Pubkey::from_str(&collection_mint)?;
    let collection_md_pubkey = derive_metadata_pda(&collection_mint_pubkey);

    let set_collection_size_ix = set_collection_size(
        metadata_program_id(),
        collection_md_pubkey,
        keypair.pubkey(),
        collection_mint_pubkey,
        None,
        size,
    );

    send_and_confirm_transaction(&client, keypair, &[set_collection_size_ix])?;

    Ok(())
}
