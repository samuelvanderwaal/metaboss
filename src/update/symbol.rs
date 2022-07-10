use super::{common::*, update_data};

pub struct UpdateSymbolAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_symbol: String,
    pub retries: u8,
}

pub struct UpdateSymbolArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_symbol: String,
}

pub async fn update_symbol_one(
    client: RpcClient,
    keypair_path: Option<String>,
    mint_account: String,
    new_symbol: String,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let old_md = decode(&client, &mint_account)?;
    let data_with_old_symbol = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_symbol.creators,
        seller_fee_basis_points: data_with_old_symbol.seller_fee_basis_points,
        name: data_with_old_symbol.name,
        symbol: new_symbol,
        uri: data_with_old_symbol.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(&client, &keypair, &mint_account, new_data)
        .map_err(|e| ActionError::ActionFailed(mint_account.to_string(), e.to_string()))?;

    Ok(())
}

pub async fn update_symbol(args: UpdateSymbolArgs) -> Result<(), ActionError> {
    let old_md = decode(&args.client, &args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
    let data_with_old_symbol = old_md.data;

    let new_data = DataV2 {
        creators: data_with_old_symbol.creators,
        seller_fee_basis_points: data_with_old_symbol.seller_fee_basis_points,
        name: data_with_old_symbol.name,
        symbol: args.new_symbol.to_owned(),
        uri: data_with_old_symbol.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(&args.client, &args.keypair, &args.mint_account, new_data)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
    Ok(())
}

pub struct UpdateSymbolAll {}

#[async_trait]
impl Action for UpdateSymbolAll {
    fn name() -> &'static str {
        "update-symbol-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        update_symbol(UpdateSymbolArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_symbol: args.new_value,
        })
        .await
    }
}

pub async fn update_symbol_all(args: UpdateSymbolAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list: args.mint_list,
        cache_file: args.cache_file,
        new_value: args.new_symbol,
        retries: args.retries,
    };
    UpdateSymbolAll::run(args).await?;

    Ok(())
}
