use super::*;

pub struct SetUpdateAuthorityAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub payer: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_authority: String,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct SetUpdateAuthorityArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub payer: Arc<Option<Keypair>>,
    pub mint_account: String,
    pub new_authority: String,
}

pub async fn set_update_authority(args: SetUpdateAuthorityArgs) -> Result<Signature, ActionError> {
    let (_current_md, token, current_rule_set) =
        update_asset_preface(&args.client, &args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    let new_authority = Pubkey::from_str(&args.new_authority)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Update the sfbp on the data struct.
    let UpdateArgs::V1 {
        ref mut new_update_authority,
        ..
    } = update_args;
    *new_update_authority = Some(new_authority);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: args.payer.as_ref().as_ref(),
        authority: &args.keypair,
        mint: args.mint_account.clone(),
        token,
        delegate_record: None::<String>, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct SetUpdateAuthorityAll {}

#[async_trait]
impl Action for SetUpdateAuthorityAll {
    fn name() -> &'static str {
        "set-update-authority-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        // Set Update Authority can have an optional payer.
        set_update_authority(SetUpdateAuthorityArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            payer: args.payer.clone(),
            mint_account: args.mint_account,
            new_authority: args.new_value,
        })
        .await
        .map(|_| ())
    }
}

pub async fn set_update_authority_all(args: SetUpdateAuthorityAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let solana_opts = parse_solana_config();
    let payer = args
        .payer
        .map(|path| parse_keypair(Some(path), solana_opts));

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list: args.mint_list,
        cache_file: args.cache_file,
        new_value: args.new_authority,
        batch_size: args.batch_size,
        retries: args.retries,
    };
    SetUpdateAuthorityAll::run(args).await
}
