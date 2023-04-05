use super::*;

pub struct UpdateNameArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_name: String,
}

pub async fn update_name(args: UpdateNameArgs) -> Result<Signature, ActionError> {
    let (mut current_md, token, _current_rule_set) =
        update_asset_preface(&args.client, &args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the name on the data struct.
    current_md.data.name = args.new_name.clone();
    let UpdateArgs::V1 { ref mut data, .. } = update_args;
    *data = Some(current_md.data);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint: args.mint_account.clone(),
        token,
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}
