use crate::cache::NewValue;

use super::*;

pub type UpdateResults = Vec<Result<(), ActionError>>;

pub struct UpdateCreatorArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_creators: String,
    pub should_append: bool,
}

pub async fn update_creator(args: UpdateCreatorArgs) -> Result<Signature, ActionError> {
    let mut current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    let parsed_creators = match parse_cli_creators(args.new_creators, args.should_append) {
        Ok(creators) => creators,
        Err(e) => return Err(ActionError::ActionFailed(args.mint_account, e.to_string())),
    };

    let new_creators = if let Some(mut old_creators) = current_md.data.creators {
        if !args.should_append {
            parsed_creators
        } else {
            let remaining_space = 5 - old_creators.len();
            warn!(
                "Appending {} new creators with old creators with shares of 0",
                parsed_creators.len()
            );
            let end_index = cmp::min(parsed_creators.len(), remaining_space);
            old_creators.append(&mut parsed_creators[0..end_index].to_vec());
            old_creators
        }
    } else {
        parsed_creators
    };

    let shares = new_creators.iter().fold(0, |acc, c| acc + c.share);
    if shares != 100 {
        return Err(ActionError::ActionFailed(
            args.mint_account,
            "Creators shares must sum to 100!".to_string(),
        ));
    }

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the creators on the data struct.
    current_md.data.creators = Some(new_creators);
    let UpdateArgs::V1 { ref mut data, .. } = update_args;
    *data = Some(current_md.data);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &args.keypair,
        mint: args.mint_account.clone(),
        token: None::<String>, // The lib will find this if it's a pNFT.
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct UpdateCreatorAllArgs {
    pub client: RpcClient,
    pub keypair_path: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_creators: String,
    pub should_append: bool,
    pub rate_limit: usize,
    pub retries: u8,
}

pub async fn update_creator_all(args: UpdateCreatorAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair_path, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::Single(args.new_creators),
        rate_limit: args.rate_limit,
        retries: args.retries,
    };
    UpdateCreatorAll::run(args).await
}

pub struct UpdateCreatorAll {}

#[async_trait]
impl Action for UpdateCreatorAll {
    fn name() -> &'static str {
        "update-creator-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        update_creator(UpdateCreatorArgs {
            client: args.client,
            keypair: args.keypair,
            mint_account: args.mint_account,
            new_creators: args.new_value,
            should_append: false,
        })
        .await
        .map(|_| ())
    }
}
