use super::common::*;

use crate::parse::parse_keypair;
use crate::{
    derive::{derive_collection_authority_record, derive_metadata_pda},
    parse::parse_solana_config,
    utils::send_and_confirm_transaction,
};
use metaboss_lib::unverify::{unverify_collection_ix, UnverifyCollectionArgs};
use metaboss_lib::update::{update_asset_ix, UpdateAssetArgs};
use metaboss_lib::verify::{verify_collection_ix, VerifyCollectionArgs};
use mpl_token_metadata::instruction::{set_collection_size, CollectionToggle, UpdateArgs};

pub const OPEN_FILES_LIMIT: usize = 1024;

pub fn set_and_verify_nft_collection(
    client: RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    _nft_auth: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let collection_pubkey = Pubkey::from_str(&collection_mint)?;

    let mut instructions = vec![];

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // We set the collection key with update, but can only verify with Verify.
    let UpdateArgs::V1 {
        ref mut collection, ..
    } = update_args;
    *collection = CollectionToggle::Set(MdCollection {
        key: collection_pubkey,
        verified: false,
    });

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &keypair,
        mint: nft_mint.clone(),
        token: None::<String>,
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
    };

    instructions.push(update_asset_ix(&client, update_args)?);

    // Add verify instruction to verify the collection.
    let verify_args = VerifyCollectionArgs::V1 {
        authority: &keypair,
        mint: nft_mint,
        collection_mint,
        is_delegate: is_delegate_present,
    };

    // This instruction handles both the case where the collection NFT exists and the case where it doesn't.
    instructions.push(verify_collection_ix(&client, verify_args)?);

    send_and_confirm_transaction(&client, keypair, &instructions)?;

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

    let unverify_args = UnverifyCollectionArgs::V1 {
        authority: &keypair,
        mint: nft_mint,
        collection_mint,
        is_delegate: is_delegate_present,
    };

    // This instruction handles both the case where the collection NFT exists and the case where it doesn't.
    let ix = unverify_collection_ix(&client, unverify_args)?;
    send_and_confirm_transaction(&client, keypair, &[ix])?;

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

    let verify_args = VerifyCollectionArgs::V1 {
        authority: &keypair,
        mint: nft_mint,
        collection_mint,
        is_delegate: is_delegate_present,
    };

    // This instruction handles both the case where the collection NFT exists and the case where it doesn't.
    let ix = verify_collection_ix(&client, verify_args)?;
    send_and_confirm_transaction(&client, keypair, &[ix])?;

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
