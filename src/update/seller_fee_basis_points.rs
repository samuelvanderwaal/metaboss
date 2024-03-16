use metaboss_lib::update::V1UpdateArgs;
use mpl_token_metadata::types::Data;

use crate::cache::NewValue;

use super::*;

pub struct UpdateSellerFeeBasisPointsArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_sfbp: u16,
    pub priority: Priority,
}
pub struct UpdateSellerFeeBasisPointsAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_sfbp: u16,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub async fn update_sfbp(args: UpdateSellerFeeBasisPointsArgs) -> Result<Signature, ActionError> {
    // Add metadata delegate record here later.
    let current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    let data = Some(Data {
        name: current_md.name,
        symbol: current_md.symbol,
        uri: current_md.uri,
        seller_fee_basis_points: args.new_sfbp,
        creators: current_md.creators,
    });

    // Token Metadata UpdateArgs enum.
    let update_args = V1UpdateArgs {
        data,
        ..Default::default()
    };

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

        update_sfbp(UpdateSellerFeeBasisPointsArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_sfbp: sfbp,
            priority: args.priority,
        })
        .await
        .map(|_| ())
    }
}

pub async fn update_sfbp_all(args: UpdateSellerFeeBasisPointsAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::Single(args.new_sfbp.to_string()),
        rate_limit: args.rate_limit,
        retries: args.retries,
        priority: args.priority,
    };
    UpdateSellerFeeBasisPointsAll::run(args).await
}
