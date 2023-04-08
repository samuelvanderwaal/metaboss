use crate::{cache::NewValue, update::parse_mint_list};

use super::*;

pub struct BurnAssetArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub token_account: Option<String>,
    pub amount: u64,
}

pub struct BurnAssetAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
}

pub async fn burn_asset(args: BurnAssetArgs) -> Result<Signature, ActionError> {
    let mint = Pubkey::from_str(&args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // We support the user passing in a non-ATA token account, but otherwise we derive the ATA.
    let token = if let Some(token) = args.token_account {
        Pubkey::from_str(&token)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?
    } else {
        get_associated_token_address(&args.keypair.pubkey(), &mint)
    };

    let burn_args = metaboss_lib::burn::BurnAssetArgs::V1 {
        authority: &args.keypair,
        mint,
        token,
        amount: args.amount,
    };

    metaboss_lib::burn::burn_asset(&args.client, burn_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct BurnAssetAll {}

#[async_trait]
impl Action for BurnAssetAll {
    fn name() -> &'static str {
        "burn-asset-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        burn_asset(BurnAssetArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account.clone(),
            token_account: None, // Must be ATA for this action, currently.
            amount: 1,
        })
        .await
        .map(|_| ())
    }
}

pub async fn burn_asset_all(args: BurnAssetAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::None,
        rate_limit: args.rate_limit,
        retries: args.retries,
    };
    BurnAssetAll::run(args).await
}
