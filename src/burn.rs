use anyhow::Result;
use metaplex_token_metadata::{id, instruction::update_metadata_accounts, state::Data};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::str::FromStr;

use crate::derive::derive_metadata_pda;
use crate::parse::parse_keypair;

pub fn burn_one(
    client: &RpcClient,
    keypair: String,
    owner_wallet: String,
    mint_address: String,
) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(&mint_address)?;
    let keypair = parse_keypair(&keypair)?;
    let owner_pubkey = Pubkey::from_str(&owner_wallet)?;

    let sig = burn(client, &keypair, &owner_pubkey, &mint_pubkey, 1)?;

    println!("{}", sig);

    Ok(())
}

pub fn burn(
    client: &RpcClient,
    signer: &Keypair,
    owner_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    amount: u64,
) -> Result<Signature> {
    let assoc = get_associated_token_address(&owner_pubkey, &mint_pubkey);
    let spl_token_program_id = spl_token::id();

    let burn_ix = spl_token::instruction::burn(
        &spl_token_program_id,
        &assoc,
        mint_pubkey,
        &signer.pubkey(),
        &[&signer.pubkey()],
        amount,
    )?;

    let close_associated_token_account = spl_token::instruction::close_account(
        &spl_token_program_id,
        &assoc,
        &signer.pubkey(),
        &signer.pubkey(),
        &[&signer.pubkey()],
    )?;

    let metadata_pubkey = derive_metadata_pda(mint_pubkey);

    let data = default_data();

    let clear_metadata_account = update_metadata_accounts(
        id(),
        metadata_pubkey,
        signer.pubkey(),
        None,
        Some(data),
        None,
    );

    let instructions = vec![
        burn_ix,
        close_associated_token_account,
        clear_metadata_account,
    ];

    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&signer.pubkey()),
        &[signer],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}

fn default_data() -> Data {
    Data {
        name: String::default(),
        symbol: String::default(),
        uri: String::default(),
        seller_fee_basis_points: u16::default(),
        creators: None,
    }
}
