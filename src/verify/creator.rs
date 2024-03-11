use super::*;

pub struct VerifyCreatorArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint: String,
}

pub struct VerifyCreatorAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
}

pub async fn verify_creator(args: VerifyCreatorArgs) -> Result<Signature, ActionError> {
    let verify_args = metaboss_lib::verify::VerifyCreatorArgs::V1 {
        authority: &args.keypair,
        mint: args.mint.clone(),
    };

    metaboss_lib::verify::verify_creator(&args.client, verify_args)
        .map_err(|e| ActionError::ActionFailed(args.mint.to_string(), e.to_string()))
}

pub struct VerifyCreatorAll {}

#[async_trait]
impl Action for VerifyCreatorAll {
    fn name() -> &'static str {
        "verify-creator-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        verify_creator(VerifyCreatorArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint: args.mint_account.clone(),
        })
        .await
        .map(|_| ())
    }
}

pub async fn verify_creator_all(args: VerifyCreatorAllArgs) -> AnyResult<()> {
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
        priority: Priority::None,
    };
    VerifyCreatorAll::run(args).await
}
