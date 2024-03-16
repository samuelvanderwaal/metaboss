use std::{collections::HashMap, fs::File};

use metaboss_lib::{data::UpdateUriData, update::V1UpdateArgs};
use mpl_token_metadata::types::Data;

use crate::cache::{MintValues, NewValue};

use super::*;

pub struct UpdateUriAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub cache_file: Option<String>,
    pub new_uris_file: String,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct UpdateUriArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_uri: String,
    pub priority: Priority,
}

pub async fn update_uri(args: UpdateUriArgs) -> Result<Signature, ActionError> {
    let current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Save a transaction by not updating if the uri is the same.
    if current_md.uri.trim_matches(char::from(0)) == args.new_uri.trim_matches(char::from(0)) {
        return Ok(Signature::default());
    }

    // Token Metadata UpdateArgs enum.
    let mut update_args = V1UpdateArgs::default();

    let data = Data {
        name: current_md.name,
        symbol: current_md.symbol,
        uri: args.new_uri,
        seller_fee_basis_points: current_md.seller_fee_basis_points,
        creators: current_md.creators,
    };

    update_args.data = Some(data);

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

pub struct UpdateUriAll {}

#[async_trait]
impl Action for UpdateUriAll {
    fn name() -> &'static str {
        "update-uri-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        update_uri(UpdateUriArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_uri: args.new_value,
            priority: args.priority,
        })
        .await
        .map(|_| ())
    }
}

pub async fn update_uri_all(args: UpdateUriAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let f = File::open(args.new_uris_file)?;
    let update_uris: Vec<UpdateUriData> = serde_json::from_reader(f)?;

    // If user hasn't passed in a cache file, then we construct the mint list from the URI file.

    let mint_list = if args.cache_file.is_none() {
        Some(
            update_uris
                .iter()
                .map(|data| data.mint_account.clone())
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let mint_values: MintValues = update_uris
        .iter()
        .map(|data| (data.mint_account.clone(), data.new_uri.clone()))
        .collect::<HashMap<_, _>>();

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::List(mint_values),
        rate_limit: args.rate_limit,
        retries: args.retries,
        priority: args.priority,
    };
    UpdateUriAll::run(args).await?;
    Ok(())
}
