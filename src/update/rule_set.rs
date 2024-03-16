use metaboss_lib::update::V1UpdateArgs;

use crate::cache::NewValue;

use super::*;

pub struct UpdateRuleSetAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_rule_set: String,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct UpdateRuleSetArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_rule_set: String,
    pub priority: Priority,
}

pub struct ClearRuleSetAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct ClearRuleSetArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub priority: Priority,
}

pub async fn update_rule_set(args: UpdateRuleSetArgs) -> Result<Signature, ActionError> {
    let new_rule_set = Pubkey::from_str(&args.new_rule_set)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let update_args = V1UpdateArgs {
        rule_set: RuleSetToggle::Set(new_rule_set),
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

pub async fn clear_rule_set(args: ClearRuleSetArgs) -> Result<Signature, ActionError> {
    let mint = Pubkey::from_str(&args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let update_args = V1UpdateArgs {
        rule_set: RuleSetToggle::Clear,
        ..Default::default()
    };

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint,
        token: None::<String>,
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
        priority: args.priority,
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
            priority: args.priority,
        })
        .await
        .map(|_| ())
    }
}

pub async fn update_rule_set_all(args: UpdateRuleSetAllArgs) -> AnyResult<()> {
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
        new_value: NewValue::Single(args.new_rule_set),
        rate_limit: args.rate_limit,
        retries: args.retries,
        priority: args.priority,
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
            priority: args.priority,
        })
        .await
        .map(|_| ())
    }
}

pub async fn clear_rule_set_all(args: ClearRuleSetAllArgs) -> AnyResult<()> {
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
    ClearRuleSetAll::run(args).await
}
