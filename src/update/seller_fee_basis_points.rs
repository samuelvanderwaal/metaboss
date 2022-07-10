use super::{common::*, update_data};

pub fn update_seller_fee_basis_points_one(
    client: &RpcClient,
    keypair: Option<String>,
    mint_account: &str,
    new_seller_fee_basis_points: &u16,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let parsed_keypair = parse_keypair(keypair, solana_opts);

    let old_md = decode(client, mint_account)?;
    let data_with_old_seller_fee_basis_points = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_seller_fee_basis_points.creators,
        seller_fee_basis_points: new_seller_fee_basis_points.to_owned(),
        name: data_with_old_seller_fee_basis_points.name,
        symbol: data_with_old_seller_fee_basis_points.symbol,
        uri: data_with_old_seller_fee_basis_points.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(client, &parsed_keypair, mint_account, new_data)?;
    Ok(())
}
