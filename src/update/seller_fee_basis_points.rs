use super::{common::*, update_data};

pub struct UpdateSellerFeeBasisPointsArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub payer: Arc<Keypair>,
    pub mint_account: String,
    pub new_sfbp: u16,
}
pub struct UpdateSellerFeeBasisPointsAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_sfbp: u16,
    pub batch_size: usize,
    pub retries: u8,
}

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

async fn update_sfbp(args: UpdateSellerFeeBasisPointsArgs) -> Result<(), ActionError> {
    let old_md = decode(&args.client, &args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
    let old_data = old_md.data;

    let new_data = DataV2 {
        creators: old_data.creators,
        seller_fee_basis_points: args.new_sfbp,
        name: old_data.name,
        symbol: old_data.symbol,
        uri: old_data.uri,
        collection: old_md.collection,
        uses: old_md.uses,
    };

    update_data(&args.client, &args.keypair, &args.mint_account, new_data)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    Ok(())
}

pub struct UpdateSellerFeeBasisPointsAll {}

#[async_trait]
impl Action for UpdateSellerFeeBasisPointsAll {
    fn name() -> &'static str {
        "update-sfbp-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        // Converting back and forth between String and u16 is dumb but I couldn't figure out a
        // nice way to do this with generics.
        let sfbp = args.new_value.parse::<u16>().map_err(|e| {
            ActionError::ActionFailed(
                args.mint_account.to_string(),
                format!("Invalid new_sfbp: {}", e),
            )
        })?;

        // Set Update Authority can have an optional payer.
        update_sfbp(UpdateSellerFeeBasisPointsArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            payer: args.payer.clone(),
            mint_account: args.mint_account,
            new_sfbp: sfbp,
        })
        .await
    }
}

pub async fn update_seller_fee_basis_points_all(
    args: UpdateSellerFeeBasisPointsAllArgs,
) -> AnyResult<()> {
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
        new_value: args.new_sfbp.to_string(),
        batch_size: args.batch_size,
        retries: args.retries,
    };
    UpdateSellerFeeBasisPointsAll::run(args).await?;

    Ok(())
}
