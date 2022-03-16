use std::str::FromStr;

use anyhow::Result;
use mpl_token_metadata::{
    id as metadata_program_id,
    instruction::{approve_use_authority, revoke_use_authority},
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

use crate::{
    derive::{derive_metadata_pda, derive_use_authority_record},
    parse::{parse_keypair, parse_solana_config},
};

fn send_and_confirm_transaction(
    client: &RpcClient,
    keypair: Keypair,
    instructions: &[Instruction],
) -> Result<String> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );

    let sig = res?;

    println!("TxId: {}", sig);
    Ok(sig.to_string())
}

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
