use anyhow::Result;
use mpl_token_metadata::state::DataV2;
use solana_client::rpc_client::RpcClient;

use crate::decode::decode;
use crate::parse::parse_keypair;
use crate::parse::parse_solana_config;

use super::update_data;

pub fn update_symbol_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_symbol: &str,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_symbol = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_symbol.creators,
        seller_fee_basis_points: data_with_old_symbol.seller_fee_basis_points,
        name: data_with_old_symbol.name,
        symbol: new_symbol.to_owned(),
        uri: data_with_old_symbol.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &keypair, mint_account, new_data)?;
    Ok(())
}
