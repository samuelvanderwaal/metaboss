use std::str::FromStr;

use anyhow::Result;
use metaboss_lib::transfer::{transfer_asset, TransferAssetArgs};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;

use crate::parse::{parse_keypair, parse_solana_config};

pub fn process_transfer_asset(
    client: &RpcClient,
    keypair_path: Option<String>,
    receiver: String,
    receiver_account: Option<String>,
    mint: String,
    amount: u64,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    // Authority is the payer as well.
    let authority = parse_keypair(keypair_path, solana_opts);
    let receiver = Pubkey::from_str(&receiver)?;
    let mint = Pubkey::from_str(&mint)?;

    let source_ata = get_associated_token_address(&authority.pubkey(), &mint);
    let destination_token = if let Some(account) = receiver_account {
        Pubkey::from_str(&account)?
    } else {
        get_associated_token_address(&receiver, &mint)
    };

    let args = TransferAssetArgs::V1 {
        payer: None,
        authority: &authority,
        mint,
        source_owner: authority.pubkey(),
        source_token: source_ata,
        destination_owner: receiver,
        destination_token,
        amount,
        authorization_data: None,
    };

    let transfer_result = transfer_asset(client, args)?;

    println!("Transferred asset: {mint:?}");
    println!("Transaction signature: {transfer_result:?}");

    Ok(())
}
