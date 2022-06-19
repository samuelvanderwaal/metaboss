use anyhow::{anyhow, Result};
use log::warn;
use mpl_token_metadata::state::DataV2;
use solana_client::rpc_client::RpcClient;
use std::cmp;

use crate::decode::decode;
use crate::parse::parse_solana_config;
use crate::parse::{parse_cli_creators, parse_keypair};

use super::update_data;

pub fn update_creator_by_position(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_creators: &str,
    should_append: bool,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_creators = old_md.data;
    let parsed_creators = parse_cli_creators(new_creators.to_string(), should_append)?;

    let new_creators = if let Some(mut old_creators) = data_with_old_creators.creators {
        if !should_append {
            parsed_creators
        } else {
            let remaining_space = 5 - old_creators.len();
            warn!(
                "Appending {} new creators with old creators with shares of 0",
                parsed_creators.len()
            );
            let end_index = cmp::min(parsed_creators.len(), remaining_space);
            old_creators.append(&mut parsed_creators[0..end_index].to_vec());
            old_creators
        }
    } else {
        parsed_creators
    };

    let shares = new_creators.iter().fold(0, |acc, c| acc + c.share);
    if shares != 100 {
        return Err(anyhow!("Creators shares must sum to 100!"));
    }

    let new_data = DataV2 {
        creators: Some(new_creators),
        seller_fee_basis_points: data_with_old_creators.seller_fee_basis_points,
        name: data_with_old_creators.name,
        symbol: data_with_old_creators.symbol,
        uri: data_with_old_creators.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &keypair, mint_account, new_data)?;
    Ok(())
}
