use std::str::FromStr;

use anyhow::Result;
use mpl_token_metadata::instructions::{
    ApproveUseAuthorityBuilder, RevokeUseAuthorityBuilder, UtilizeBuilder,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::{
    derive::{derive_metadata_pda, derive_use_authority_record},
    parse::{parse_keypair, parse_solana_config},
    utils::send_and_confirm_transaction,
};

pub fn approve_use_delegate(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    use_auth_delegate: String,
    owner_nft_token_account: String,
    burner_program_id: String,
    number_of_uses: u64,
) -> Result<()> {
    let nft_pubkey = Pubkey::from_str(&nft_mint)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let delegate_pubkey = Pubkey::from_str(&use_auth_delegate)?;
    let owner_nft_token_pubkey = Pubkey::from_str(&owner_nft_token_account)?;
    let burner_program_pubkey = Pubkey::from_str(&burner_program_id)?;

    let (use_authority_record, _bump) = derive_use_authority_record(&nft_pubkey, &delegate_pubkey);

    let nft_metadata = derive_metadata_pda(&nft_pubkey);

    let approve_use_auth_ix = ApproveUseAuthorityBuilder::new()
        .use_authority_record(use_authority_record)
        .metadata(nft_metadata)
        .owner(keypair.pubkey())
        .payer(keypair.pubkey())
        .mint(nft_pubkey)
        .burner(burner_program_pubkey)
        .owner_token_account(owner_nft_token_pubkey)
        .user(delegate_pubkey)
        .number_of_uses(number_of_uses)
        .instruction();

    send_and_confirm_transaction(client, keypair, &[approve_use_auth_ix])?;

    Ok(())
}

pub fn revoke_use_delegate(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    use_auth_delegate: String,
    owner_nft_token_account: String,
) -> Result<()> {
    let nft_pubkey = Pubkey::from_str(&nft_mint)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let delegate_pubkey = Pubkey::from_str(&use_auth_delegate)?;
    let owner_nft_token_pubkey = Pubkey::from_str(&owner_nft_token_account)?;

    let (use_authority_record, _bump) = derive_use_authority_record(&nft_pubkey, &delegate_pubkey);

    let nft_metadata = derive_metadata_pda(&nft_pubkey);

    let revoke_use_auth_ix = RevokeUseAuthorityBuilder::new()
        .use_authority_record(use_authority_record)
        .user(delegate_pubkey)
        .owner(keypair.pubkey())
        .owner_token_account(owner_nft_token_pubkey)
        .metadata(nft_metadata)
        .mint(nft_pubkey)
        .instruction();

    send_and_confirm_transaction(client, keypair, &[revoke_use_auth_ix])?;

    Ok(())
}

pub fn utilize_nft(
    client: &RpcClient,
    keypair_path: Option<String>,
    nft_mint: String,
    nft_owner: String,
    owner_nft_token_account: String,
    burner_program_id: Option<String>,
    is_delegate_present: bool,
) -> Result<()> {
    let nft_pubkey = Pubkey::from_str(&nft_mint)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let nft_owner = Pubkey::from_str(&nft_owner)?;
    let owner_nft_token_pubkey = Pubkey::from_str(&owner_nft_token_account)?;
    let delegate_pubkey = keypair.pubkey();
    let nft_metadata = derive_metadata_pda(&nft_pubkey);

    let use_authority_record = match is_delegate_present {
        true => Some(derive_use_authority_record(&nft_pubkey, &delegate_pubkey).0),
        false => None,
    };

    let burner_program_pubkey = if let Some(burner_program_id) = burner_program_id {
        Some(Pubkey::from_str(&burner_program_id)?)
    } else {
        None
    };

    let mut builder = UtilizeBuilder::new();
    builder
        .metadata(nft_metadata)
        .token_account(owner_nft_token_pubkey)
        .mint(nft_pubkey)
        .owner(nft_owner)
        .number_of_uses(1);

    if let Some(use_authority_record) = use_authority_record {
        builder.use_authority_record(Some(use_authority_record));
    }
    if let Some(burner_program_pubkey) = burner_program_pubkey {
        builder.burner(Some(burner_program_pubkey));
    }
    let utilize_nft_ix = builder.instruction();

    send_and_confirm_transaction(client, keypair, &[utilize_nft_ix])?;

    Ok(())
}
