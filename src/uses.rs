use std::str::FromStr;

use anyhow::Result;
use mpl_token_metadata::{
    id as metadata_program_id,
    instruction::{approve_use_authority, revoke_use_authority, utilize},
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

    let approve_use_auth_ix = approve_use_authority(
        metadata_program_id(),
        use_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        keypair.pubkey(),
        owner_nft_token_pubkey,
        nft_metadata,
        nft_pubkey,
        burner_program_pubkey,
        number_of_uses,
    );

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

    let revoke_use_auth_ix = revoke_use_authority(
        metadata_program_id(),
        use_authority_record,
        delegate_pubkey,
        keypair.pubkey(),
        owner_nft_token_pubkey,
        nft_metadata,
        nft_pubkey,
    );

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

    let utilize_nft_ix = utilize(
        metadata_program_id(),
        nft_metadata,
        owner_nft_token_pubkey,
        nft_pubkey,
        use_authority_record,
        delegate_pubkey,
        nft_owner,
        burner_program_pubkey,
        1,
    );

    send_and_confirm_transaction(client, keypair, &[utilize_nft_ix])?;

    Ok(())
}
