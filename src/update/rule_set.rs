use super::*;

pub struct UpdateRuleSetAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_rule_set: String,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct UpdateRuleSetArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_rule_set: String,
}

pub struct ClearRuleSetAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct ClearRuleSetArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
}

pub async fn update_rule_set(args: UpdateRuleSetArgs) -> Result<Signature, ActionError> {
    let md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // We need the token account passed in for pNFT updates.
    let token = Some(
        get_nft_token_account(&args.client, &args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?,
    );

    let new_rule_set = Pubkey::from_str(&args.new_rule_set)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the rule set.
    let UpdateArgs::V1 {
        ref mut rule_set, ..
    } = update_args;

    *rule_set = RuleSetToggle::Set(new_rule_set);

    let current_rule_set = if let Some(ProgrammableConfig::V1 { rule_set }) = md.programmable_config
    {
        rule_set
    } else {
        None
    };

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
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

pub async fn clear_rule_set(args: ClearRuleSetArgs) -> Result<Signature, ActionError> {
    let md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // We need the token account passed in for pNFT updates.
    let token = Some(
        get_nft_token_account(&args.client, &args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?,
    );

    let mint = Pubkey::from_str(&args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the rule set.
    let UpdateArgs::V1 {
        ref mut rule_set, ..
    } = update_args;

    *rule_set = RuleSetToggle::Clear;

    let current_rule_set = if let Some(ProgrammableConfig::V1 { rule_set }) = md.programmable_config
    {
        rule_set
    } else {
        None
    };

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint,
        token,
        delegate_record: None::<String>, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct UpdateRuleSetAll {}

#[async_trait]
impl Action for UpdateRuleSetAll {
    fn name() -> &'static str {
        "update-rule-set-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        update_rule_set(UpdateRuleSetArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_rule_set: args.new_value,
        })
        .await
        .map(|_| ())
    }
}

pub async fn update_rule_set_all(args: UpdateRuleSetAllArgs) -> AnyResult<()> {
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
        new_value: args.new_rule_set,
        batch_size: args.batch_size,
        retries: args.retries,
    };
    UpdateRuleSetAll::run(args).await
}

pub struct ClearRuleSetAll {}

#[async_trait]
impl Action for ClearRuleSetAll {
    fn name() -> &'static str {
        "clear-rule-set-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        clear_rule_set(ClearRuleSetArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
        })
        .await
        .map(|_| ())
    }
}

pub async fn clear_rule_set_all(args: ClearRuleSetAllArgs) -> AnyResult<()> {
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
        new_value: "".to_string(),
        batch_size: args.batch_size,
        retries: args.retries,
    };
    ClearRuleSetAll::run(args).await
}
