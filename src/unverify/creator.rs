use super::*;

pub struct UnverifyCreatorArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint: String,
}

pub struct UnverifyCreatorAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub async fn unverify_creator(args: UnverifyCreatorArgs) -> Result<Signature, ActionError> {
    let verify_args = metaboss_lib::unverify::UnverifyCreatorArgs::V1 {
        authority: &args.keypair,
        mint: args.mint.clone(),
    };

    metaboss_lib::unverify::unverify_creator(&args.client, verify_args)
        .map_err(|e| ActionError::ActionFailed(args.mint.to_string(), e.to_string()))
}

pub struct UnverifyCreatorAll {}

#[async_trait]
impl Action for UnverifyCreatorAll {
    fn name() -> &'static str {
        "unverify-creator-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        unverify_creator(UnverifyCreatorArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint: args.mint_account.clone(),
        })
        .await
        .map(|_| ())
    }
}

pub async fn unverify_creator_all(args: UnverifyCreatorAllArgs) -> AnyResult<()> {
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
        batch_size: args.batch_size,
        retries: args.retries,
    };
    UnverifyCreatorAll::run(args).await
}
