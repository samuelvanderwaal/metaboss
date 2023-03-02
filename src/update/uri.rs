use std::fs::File;

use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

use crate::{constants::USE_RATE_LIMIT, data::UpdateUriData, limiter::create_default_rate_limiter};

use super::*;

pub struct UpdateUriAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub payer: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub new_uri: String,
    pub batch_size: usize,
    pub retries: u8,
}

pub struct UpdateUriArgs<'a> {
    pub client: &'a RpcClient,
    pub keypair: &'a Keypair,
    pub payer: Option<&'a Keypair>,
    pub mint_account: String,
    pub new_uri: String,
}

pub fn update_uri(args: UpdateUriArgs) -> Result<Signature, ActionError> {
    let (mut current_md, token, current_rule_set) =
        update_asset_preface(args.client, &args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    // Add metadata delegate record here later.

    // Save a transaction by not updating if the uri is the same.
    if current_md.data.uri.trim_matches(char::from(0)) == args.new_uri.trim_matches(char::from(0)) {
        return Ok(Signature::default());
    }

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    current_md.data.uri = args.new_uri;
    let UpdateArgs::V1 { ref mut data, .. } = update_args;
    *data = Some(current_md.data);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: args.keypair,
        mint: args.mint_account.clone(),
        token,
        delegate_record: None::<String>, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    update_asset(args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub fn update_uri_all(
    client: &RpcClient,
    keypair_path: Option<String>,
    json_file: &str,
) -> AnyResult<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_default_rate_limiter();

    let solana_opts = parse_solana_config();
    let keypair = &parse_keypair(keypair_path, solana_opts);

    let f = File::open(json_file)?;
    let update_uris: Vec<UpdateUriData> = serde_json::from_reader(f)?;

    update_uris.par_iter().for_each(|data| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        let args = UpdateUriArgs {
            client,
            keypair,
            payer: None,
            mint_account: data.mint_account.clone(),
            new_uri: data.new_uri.clone(),
        };

        match update_uri(args) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to update uri: {:?} error: {}", data, e);
            }
        }
    });

    Ok(())
}
