use super::*;

use crate::parse::parse_keypair;
use crate::{parse::parse_solana_config, utils::send_and_confirm_transaction};
use metaboss_lib::derive::derive_metadata_pda;
use metaboss_lib::update::V1UpdateArgs;
use metaboss_lib::{
    delegate::{delegate_asset, DelegateAssetArgs},
    revoke::{revoke_asset, RevokeAssetArgs},
    unverify::{unverify_collection_ix, UnverifyCollectionArgs},
    update::{update_asset_ix, UpdateAssetArgs},
    verify::{verify_collection_ix, VerifyCollectionArgs},
};
use mpl_token_metadata::types::SetCollectionSizeArgs;
use mpl_token_metadata::{
    instructions::SetCollectionSizeBuilder,
    types::{CollectionToggle, DelegateArgs, RevokeArgs},
};

pub const OPEN_FILES_LIMIT: usize = 1024;

pub fn set_and_verify_nft_collection(
    client: RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    collection_mint: String,
    is_delegate_present: bool,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let collection_pubkey = Pubkey::from_str(&collection_mint)?;

    let mut instructions = vec![];

    // Token Metadata UpdateArgs enum.
    let update_args = V1UpdateArgs {
        collection: CollectionToggle::Set(MdCollection {
            key: collection_pubkey,
            verified: false,
        }),
        ..Default::default()
    };

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
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let delegate_args = DelegateAssetArgs::V1 {
        payer: None,
        authority: &keypair,
        mint: collection_mint,
        delegate: delegate_pubkey,
        token: None::<String>,
        delegate_args: DelegateArgs::CollectionV1 {
            authorization_data: None,
        },
    };

    let sig = delegate_asset(&client, delegate_args)?;

    println!("Signature: {}", sig);

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

    let delegate_pubkey = Pubkey::from_str(&delegate_authority)?;

    let revoke_args = RevokeAssetArgs::V1 {
        payer: None,
        authority: &keypair,
        mint: collection_mint,
        delegate: delegate_pubkey,
        token: None::<String>,
        revoke_args: RevokeArgs::CollectionV1,
    };

    let sig = revoke_asset(&client, revoke_args)?;

    println!("Signature: {}", sig);

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

    let set_collection_size_ix = SetCollectionSizeBuilder::new()
        .collection_metadata(collection_md_pubkey)
        .collection_authority(keypair.pubkey())
        .collection_mint(collection_mint_pubkey)
        .set_collection_size_args(SetCollectionSizeArgs { size })
        .instruction();

    send_and_confirm_transaction(&client, keypair, &[set_collection_size_ix])?;

    Ok(())
}
