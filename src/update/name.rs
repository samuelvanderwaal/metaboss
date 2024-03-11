use metaboss_lib::update::V1UpdateArgs;
use mpl_token_metadata::types::Data;

use super::*;

pub struct UpdateNameArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_name: String,
    pub priority: Priority,
}

pub async fn update_name(args: UpdateNameArgs) -> Result<Signature, ActionError> {
    let current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Token Metadata UpdateArgs enum.
    let mut update_args = V1UpdateArgs::default();

    let data = Data {
        name: args.new_name,
        symbol: current_md.symbol,
        uri: current_md.uri,
        seller_fee_basis_points: current_md.seller_fee_basis_points,
        creators: current_md.creators,
    };

    update_args.data = Some(data);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint: args.mint_account.clone(),
        token: None::<String>,
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
        priority: args.priority,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}
