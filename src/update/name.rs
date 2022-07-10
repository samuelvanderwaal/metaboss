use super::{common::*, update_data};

pub fn update_name_one(
    client: &RpcClient,
    keypair: Option<String>,
    mint_account: &str,
    new_name: &str,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let parsed_keypair = parse_keypair(keypair, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_name = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_name.creators,
        seller_fee_basis_points: data_with_old_name.seller_fee_basis_points,
        name: new_name.to_owned(),
        symbol: data_with_old_name.symbol,
        uri: data_with_old_name.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &parsed_keypair, mint_account, new_data)?;
    Ok(())
}
