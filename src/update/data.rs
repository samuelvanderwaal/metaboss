use glob::glob;
use indicatif::ParallelProgressIterator;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, path::Path, sync::Mutex};

use crate::{cache::NewValue, data::UpdateNftData};

use super::*;

pub struct UpdateDataAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub cache_file: Option<String>,
    pub new_data_dir: String,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct UpdateDataArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub new_data: Data,
}

pub async fn update_data(args: UpdateDataArgs) -> Result<Signature, ActionError> {
    let (_current_md, token, current_rule_set) =
        update_asset_preface(&args.client, &args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    let UpdateArgs::V1 { ref mut data, .. } = update_args;
    *data = Some(args.new_data);

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

pub struct UpdateDataAll {}

#[async_trait]
impl Action for UpdateDataAll {
    fn name() -> &'static str {
        "update-data-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        let new_data: Data = serde_json::from_str(&args.new_value).map_err(|e| {
            ActionError::ActionFailed(
                args.mint_account.to_string(),
                format!("Failed to parse new data: {}", e),
            )
        })?;

        update_data(UpdateDataArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account,
            new_data,
        })
        .await
        .map(|_| ())
    }
}

pub async fn update_data_all(args: UpdateDataAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let path = Path::new(&args.new_data_dir).join("*.json");
    let pattern = path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid directory path"))?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.partition(Result::is_ok);

    if !errors.is_empty() {
        for error in errors {
            error!("Failed to read path: {:?}", error);
            println!("Failed to read path: {:?}", error);
        }
        return Err(anyhow!("Failed to read paths"));
    }

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();

    let mint_values = Arc::new(Mutex::new(HashMap::new()));

    // If user hasn't passed in a cache file, then we construct the mint list from the URI file.

    info!("Updating...");
    println!("Updating...");
    paths.par_iter().progress().for_each(|path| {
        let f = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open file: {:?} error: {}", path, e);
                println!("Failed to open file: {:?} error: {}", path, e);
                return;
            }
        };

        let update_nft_data: UpdateNftData = match serde_json::from_reader(f) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Failed to parse JSON data from file: {:?} error: {}",
                    path, e
                );
                println!(
                    "Failed to parse JSON data from file: {:?} error: {}",
                    path, e
                );
                return;
            }
        };

        mint_values.lock().unwrap().insert(
            update_nft_data.mint,
            serde_json::to_string(&update_nft_data.data).unwrap(),
        );
    });

    let mint_list = if args.cache_file.is_none() {
        Some(
            mint_values
                .lock()
                .unwrap()
                .iter()
                .map(|data| data.0.clone())
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let mint_values = mint_values.lock().unwrap().clone();

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::List(mint_values),
        batch_size: args.batch_size,
        retries: args.retries,
    };
    UpdateDataAll::run(args).await
}
