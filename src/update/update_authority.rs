use super::common::*;

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
    pub payer: Arc<Keypair>,
    pub mint_account: String,
    pub new_authority: String,
}

pub fn set_update_authority_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint_account: &str,
    new_update_authority: &str,
    keypair_payer_path: Option<String>,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path.clone(), solana_opts);

    let solana_opts = parse_solana_config();
    let keypair_payer = parse_keypair(keypair_payer_path.or(keypair_path), solana_opts);

    let mint_pubkey = Pubkey::from_str(mint_account)?;
    let update_authority = keypair.pubkey();
    let new_update_authority = Pubkey::from_str(new_update_authority)?;

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts_v2(
        TOKEN_METADATA_PROGRAM_ID,
        metadata_account,
        update_authority,
        Some(new_update_authority),
        None,
        None,
        None,
    );
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair_payer.pubkey()),
        &[&keypair, &keypair_payer],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    info!("Tx sig: {:?}", sig);
    println!("Tx sig: {:?}", sig);

    Ok(())
}

async fn set_update_authority(args: SetUpdateAuthorityArgs) -> Result<(), ActionError> {
    let mint_pubkey = Pubkey::from_str(&args.mint_account).expect("Invalid mint account");
    let update_authority = args.keypair.pubkey();
    let new_update_authority =
        Pubkey::from_str(&args.new_authority).expect("Invalid new update authority");

    let metadata_account = get_metadata_pda(mint_pubkey);

    let ix = update_metadata_accounts_v2(
        TOKEN_METADATA_PROGRAM_ID,
        metadata_account,
        update_authority,
        Some(new_update_authority),
        None,
        None,
        None,
    );
    let recent_blockhash = args
        .client
        .get_latest_blockhash()
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&args.keypair.pubkey()),
        &[&*args.keypair, &*args.payer],
        recent_blockhash,
    );

    let sig = args
        .client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
    info!("Tx sig: {:?}", sig);

    Ok(())
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
    SetUpdateAuthorityAll::run(args).await?;

    Ok(())
}
