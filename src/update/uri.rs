use crate::data::UpdateUriData;
use crate::decode::{decode, get_metadata_pda};
use crate::limiter::create_default_rate_limiter;
use crate::parse::parse_keypair;
use crate::{constants::*, parse::parse_solana_config};
use anyhow::Result;
use log::{error, info};
use mpl_token_metadata::{instruction::update_metadata_accounts_v2, state::DataV2};
use rayon::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use std::{fs::File, str::FromStr};

pub fn update_uri_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_uri: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    update_uri(client, &keypair, mint_account, new_uri)?;

    Ok(())
}

pub fn update_uri_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    json_file: &str,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let f = File::open(json_file)?;
    let update_uris: Vec<UpdateUriData> = serde_json::from_reader(f)?;

    update_uris.par_iter().for_each(|data| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        match update_uri(client, &keypair, &data.mint_account, &data.new_uri) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update uri: {:?} error: {}", data, e);
            }
        }
    });

    Ok(())
}

pub fn update_uri(
    client: &RpcClient,
    keypair: &Keypair,
    mint_account: &str,
    new_uri: &str,
) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let update_authority = keypair.pubkey();

    let metadata_account = get_metadata_pda(mint_pubkey);
    let metadata = decode(client, mint_account)?;

    let mut data = metadata.data;
    if data.uri.trim_matches(char::from(0)) != new_uri.trim_matches(char::from(0)) {
        data.uri = new_uri.to_string();

        let data_v2 = DataV2 {
            name: data.name,
            symbol: data.symbol,
            uri: data.uri,
            seller_fee_basis_points: data.seller_fee_basis_points,
            creators: data.creators,
            collection: metadata.collection,
            uses: metadata.uses,
        };

        let ix = update_metadata_accounts_v2(
            program_id,
            metadata_account,
            update_authority,
            None,
            Some(data_v2),
            None,
            None,
        );

        let recent_blockhash = client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&update_authority),
            &[keypair],
            recent_blockhash,
        );

        let sig = client.send_and_confirm_transaction(&tx)?;
        info!("Tx sig: {:?}", sig);
        println!("Tx sig: {sig:?}");
    } else {
        println!("URI is the same.");
    }

    Ok(())
}
