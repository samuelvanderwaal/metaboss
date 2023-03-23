use super::*;

pub struct VerifyCollectionArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint: String,
    pub collection_mint: String,
    pub is_delegate: bool,
}

pub struct VerifyCollectionAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub async fn verify_collection(args: VerifyCollectionArgs) -> Result<Signature, ActionError> {
    // let mint_pubkey = Pubkey::from_str(&args.mint).map_err(|e| {
    //     ActionError::ActionFailed(
    //         args.mint.to_string(),
    //         format!("Failed to parse mint as pubkey: {}", e),
    //     )
    // })?;
    // let asset = Asset::new(mint_pubkey);

    // let md = asset.get_metadata(&args.client).map_err(|e| {
    //     ActionError::ActionFailed(
    //         args.mint.to_string(),
    //         format!("Failed to get metadata: {}", e),
    //     )
    // })?;

    let verify_args = metaboss_lib::verify::VerifyCollectionArgs::V1 {
        authority: &args.keypair,
        mint: args.mint.clone(),
        collection_mint: args.collection_mint.clone(),
        is_delegate: args.is_delegate,
    };

    metaboss_lib::verify::verify_collection(&args.client, verify_args)
        .map_err(|e| ActionError::ActionFailed(args.mint.to_string(), e.to_string()))
}

// pub struct VerifyCollectionAll {}

// #[async_trait]
// impl Action for VerifyCollectionAll {
//     fn name() -> &'static str {
//         "verify-collection-all"
//     }

//     async fn action(args: RunActionArgs) -> Result<(), ActionError> {
//         verify_collection(VerifyCollectionArgs {
//             client: args.client.clone(),
//             keypair: args.keypair.clone(),
//             mint: args.mint_account.clone(),
//         })
//         .await
//         .map(|_| ())
//     }
// }

// pub async fn verify_collection_all(args: VerifyCollectionAllArgs) -> AnyResult<()> {
//     let solana_opts = parse_solana_config();
//     let keypair = parse_keypair(args.keypair, solana_opts);

//     let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

//     // We don't support an optional payer for this action currently.
//     let payer = None;

//     let args = BatchActionArgs {
//         client: args.client,
//         keypair,
//         payer,
//         mint_list,
//         cache_file: args.cache_file,
//         new_value: NewValue::None,
//         batch_size: args.batch_size,
//         retries: args.retries,
//     };
//     VerifyCollectionAll::run(args).await
// }
