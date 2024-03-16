use metaboss_lib::update::V1UpdateArgs;

use crate::cache::NewValue;

use super::*;

pub struct SetPrimarySaleHappenedAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct SetPrimarySaleHappenedArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub priority: Priority,
}

pub async fn set_primary_sale_happened(
    args: SetPrimarySaleHappenedArgs,
) -> Result<Signature, ActionError> {
    // Token Metadata UpdateArgs enum.
    let update_args = V1UpdateArgs {
        primary_sale_happened: Some(true),
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

pub struct SetPrimarySaleHappenedAll {}

#[async_trait]
impl Action for SetPrimarySaleHappenedAll {
    fn name() -> &'static str {
        "set-secondary-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        set_primary_sale_happened(SetPrimarySaleHappenedArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            priority: args.priority,
        })
        .await
        .map(|_| ())
    }
}

pub async fn set_primary_sale_happened_all(args: SetPrimarySaleHappenedAllArgs) -> AnyResult<()> {
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
        new_value: NewValue::None,
        rate_limit: args.rate_limit,
        retries: args.retries,
        priority: args.priority,
    };
    SetPrimarySaleHappenedAll::run(args).await
}
