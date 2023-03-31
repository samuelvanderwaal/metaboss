use crate::cache::NewValue;

use super::*;

pub struct SetUpdateAuthorityAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub payer: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_authority: String,
    pub rate_limit: usize,
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
        token: None::<String>,
        delegate_record: None::<String>, // Not supported yet in update.
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

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    let solana_opts = parse_solana_config();
    let payer = args
        .payer
        .map(|path| parse_keypair(Some(path), solana_opts));

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::Single(args.new_authority),
        rate_limit: args.rate_limit,
        retries: args.retries,
    };
    SetUpdateAuthorityAll::run(args).await
}
