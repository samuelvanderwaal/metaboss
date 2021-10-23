use std::str::FromStr;

use anyhow::{anyhow, Result};
use metaplex_token_metadata::{
    instruction::sign_metadata, state::Metadata, ID as METAPLEX_PROGRAM_ID,
};
use solana_client::rpc_client::RpcClient;
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};

use crate::parse::parse_keypair;
use crate::snapshot::get_cm_creator_accounts;

pub fn sign(
    client: &RpcClient,
    keypair: &String,
    candy_machine_id: &String,
    mint: &String,
) -> Result<()> {
    let keypair = parse_keypair(keypair)?;

    if !mint.is_empty() {
        let public_key = match Pubkey::from_str(mint) {
            Ok(f) => f,
            Err(_) => return Err(anyhow!("Invalid mint public key: {}", mint)),
        };
        print!("pk: {}", public_key);
        let ix = sign_metadata(METAPLEX_PROGRAM_ID, public_key, keypair.pubkey());
        let (recent_blockhash, _) = client.get_recent_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );
        let sig = client.send_and_confirm_transaction(&tx)?;
        println!("{}", sig);
        return Ok(());
    }

    let accounts = get_cm_creator_accounts(client, candy_machine_id)?;
    let mut accounts_to_sign = Vec::new();

    for (pubkey, account) in &accounts {
        // let creators = get_creators_metadata(account.data.clone())?;
        let metadata: Metadata = try_from_slice_unchecked(&account.data.clone())?;
        if let Some(creators) = metadata.data.creators {
            let mut verified = true;
            for creator in creators {
                if !creator.verified {
                    println!("Found creator unverified for account: {}", pubkey);
                    verified = false;
                }
            }

            if !verified {
                accounts_to_sign.push((pubkey.clone(), account.clone()));
            }
        } else {
            // No creators for that token, nothing to sign.
            continue;
        }
    }

    if accounts_to_sign.is_empty() {
        println!("No unverified metadata for this creator and candy machine.");
        return Ok(());
    }

    println!("Signing all unverified metadata...");
    for (pubkey, _) in &accounts_to_sign {
        let (recent_blockhash, _) = client.get_recent_blockhash()?;
        let ix = sign_metadata(METAPLEX_PROGRAM_ID, *pubkey, keypair.pubkey());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );
        let sig = client.send_and_confirm_transaction(&tx)?;
        println!("{}", sig);
    }

    Ok(())
}
