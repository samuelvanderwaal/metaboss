use anyhow::Result;
use mpl_token_metadata::state::{DataV2, UseMethod, Uses};
use solana_client::rpc_client::RpcClient;

use super::{
    common::{decode, parse_keypair, parse_solana_config, ActionError},
    update_data,
};

pub struct UsesArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub account: String,
    pub method: String,
    pub remaining: u64,
    pub total: u64,
    pub overwrite: bool,
}

pub fn update_uses_one(args: UsesArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let old_md = decode(&args.client, &args.account)?;
    let old_data = old_md.data;

    let use_method = match args.method.to_lowercase().as_str() {
        "burn" => UseMethod::Burn,
        "multiple" => UseMethod::Multiple,
        "single" => UseMethod::Single,
        _ => {
            println!("Invalid Uses method! Must be one of: burn, multiple, single");
            return Ok(());
        }
    };

    let new_uses = Uses {
        use_method,
        remaining: args.remaining,
        total: args.total,
    };

    // Only overwrite existing uses if the override flag is set
    if old_md.uses.is_some() && !args.overwrite {
        println!("Uses already exist for this token. Use the --overwrite flag to overwrite.");
        return Ok(());
    }

    let new_data = DataV2 {
        creators: old_data.creators,
        seller_fee_basis_points: old_data.seller_fee_basis_points,
        name: old_data.name,
        symbol: old_data.symbol,
        uri: old_data.uri,
        collection: old_md.collection,
        uses: Some(new_uses),
    };

    update_data(&args.client, &keypair, &args.account, new_data)
        .map_err(|e| ActionError::ActionFailed(args.account.to_string(), e.to_string()))?;

    Ok(())
}
