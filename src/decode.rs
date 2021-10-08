use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::pubkey::Pubkey;
use spl_token_metadata::state::Metadata;
use std::fs::File;
use std::{process, str::FromStr};

use crate::constants::*;

#[derive(Debug, Serialize)]
pub struct JSONCreator {
    pub address: String,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

pub fn decode_metadata(
    client: RpcClient,
    mint_accounts: &Vec<String>,
    output: &String,
) -> Result<()> {
    for account in mint_accounts {
        let pubkey = Pubkey::from_str(&account)?;
        let metadata_pda = match get_metadata_pda(pubkey) {
            Some(pubkey) => pubkey,
            None => panic!("No metaplex account found"),
        };
        println!("Metadata Account: {}", metadata_pda);

        let account_data = match client.get_account_data(&metadata_pda) {
            Ok(data) => data,
            Err(_) => {
                println!("No account data found! Are you on the right network?");
                process::exit(1);
            }
        };

        let metadata: Metadata = try_from_slice_unchecked(&account_data)?;

        let mut creators: Vec<JSONCreator> = Vec::new();

        if let Some(c) = metadata.data.creators {
            creators = c
                .iter()
                .map(|c| JSONCreator {
                    address: c.address.to_string(),
                    verified: c.verified,
                    share: c.share,
                })
                .collect::<Vec<JSONCreator>>();
        }

        let nft_metadata = json!({
            "name": metadata.data.name.to_string().trim_matches(char::from(0)),
            "symbol": metadata.data.symbol.to_string().trim_matches(char::from(0)),
            "seller_fee_basis_points": metadata.data.seller_fee_basis_points,
            "uri": metadata.data.uri.to_string().trim_matches(char::from(0)),
            "creators": [creators],
        });
        let mut file = File::create(format!("{}/{}.json", output, account))?;
        serde_json::to_writer(&mut file, &nft_metadata)?;
    }
    Ok(())
}

fn get_metadata_pda(pubkey: Pubkey) -> Option<Pubkey> {
    let metaplex_pubkey = METAPLEX_PROGRAM_ID
        .parse::<Pubkey>()
        .expect("Failed to parse Metaplex Program Id");

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    Some(pda)
}
