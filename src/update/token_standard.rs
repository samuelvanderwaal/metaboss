use solana_sdk::commitment_config::CommitmentConfig;

use super::*;

pub struct SetTokenStandardArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
}

pub struct SetTokenStandardAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub async fn set_token_standard_one(args: SetTokenStandardArgs) -> Result<Signature, ActionError> {
    let mint_pubkey = Pubkey::from_str(&args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    let update_authority = args.keypair.pubkey();
    let metadata_pubkey = derive_metadata_pda(&mint_pubkey);
    let edition_pubkey = derive_edition_pda(&mint_pubkey);

    let edition_opt = args
        .client
        .get_account_with_commitment(&edition_pubkey, CommitmentConfig::confirmed())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.clone(), e.to_string()))?
        .value
        .map(|_| edition_pubkey);

    let ix = set_token_standard(
        TOKEN_METADATA_PROGRAM_ID,
        metadata_pubkey,
        update_authority,
        mint_pubkey,
        edition_opt,
    );

    let recent_blockhash = args
        .client
        .get_latest_blockhash()
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&update_authority),
        &[&*args.keypair],
        recent_blockhash,
    );

    args.client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}

pub struct SetTokenStandardAll {}

#[async_trait]
impl Action for SetTokenStandardAll {
    fn name() -> &'static str {
        "set-token-standard-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        set_token_standard_one(SetTokenStandardArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_account: args.mint_account.clone(),
        })
        .await
        .map(|_| ())
    }
}

pub async fn set_token_standard_all(args: SetTokenStandardAllArgs) -> AnyResult<()> {
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
    SetTokenStandardAll::run(args).await
}
