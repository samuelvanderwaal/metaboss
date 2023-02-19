use mpl_token_metadata::state::TokenStandard;

use super::*;

pub struct UpdateSellerFeeBasisPointsArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
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

pub async fn update_sfbp(args: UpdateSellerFeeBasisPointsArgs) -> Result<(), ActionError> {
    let mut current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // We need the token account passed in for pNFT updates.
    let token = if let Some(TokenStandard::ProgrammableNonFungible) = current_md.token_standard {
        Some(
            get_nft_token_account(&args.client, &args.mint_account).map_err(|e| {
                ActionError::ActionFailed(args.mint_account.to_string(), e.to_string())
            })?,
        )
    } else {
        None
    };

    let mint = Pubkey::from_str(&args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    current_md.data.seller_fee_basis_points = args.new_sfbp;
    let current_rule_set =
        if let Some(ProgrammableConfig::V1 { rule_set }) = current_md.programmable_config {
            rule_set
        } else {
            None
        };

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the sfbp on the data struct.
    let UpdateArgs::V1 { ref mut data, .. } = update_args;
    *data = Some(current_md.data);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint,
        token,
        delegate_record: None, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    let _sig = update_asset(&args.client, update_args)
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
                format!("Invalid new_sfbp: {e}"),
            )
        })?;

        // Set Update Authority can have an optional payer.
        update_sfbp(UpdateSellerFeeBasisPointsArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_sfbp: sfbp,
        })
        .await
    }
}

pub async fn update_sfbp_all(args: UpdateSellerFeeBasisPointsAllArgs) -> AnyResult<()> {
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
