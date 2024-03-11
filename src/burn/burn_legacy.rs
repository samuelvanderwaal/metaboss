use borsh::BorshDeserialize;

use crate::{cache::NewValue, update::parse_mint_list};

use super::*;

pub struct BurnAll {}

pub struct BurnPrintAll {}

pub struct BurnAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct BurnPrintAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub master_mint: String,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub struct BurnArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_pubkey: Pubkey,
    pub priority: Priority,
}

pub struct BurnPrintArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_pubkey: Pubkey,
    pub master_mint_pubkey: Pubkey,
    pub priority: Priority,
}

pub async fn burn_one(
    client: RpcClient,
    keypair: Option<String>,
    mint_address: String,
    priority: Priority,
) -> AnyResult<()> {
    let mint_pubkey = Pubkey::from_str(&mint_address)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair, solana_opts);

    let client = Arc::new(client);
    let keypair = Arc::new(keypair);

    let args = BurnArgs {
        client,
        keypair,
        mint_pubkey,
        priority,
    };

    let sig = burn(args).await?;

    println!("Tx sig: {sig}");

    Ok(())
}

pub async fn burn_print_one(
    client: RpcClient,
    keypair: Option<String>,
    mint_address: String,
    master_mint_address: String,
    priority: Priority,
) -> AnyResult<()> {
    let mint_pubkey = Pubkey::from_str(&mint_address)?;
    let master_mint_pubkey = Pubkey::from_str(&master_mint_address)?;
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair, solana_opts);

    let client = Arc::new(client);
    let keypair = Arc::new(keypair);

    let args = BurnPrintArgs {
        client,
        keypair,
        mint_pubkey,
        master_mint_pubkey,
        priority,
    };

    let sig = burn_print(args).await?;

    println!("Tx sig: {sig}");

    Ok(())
}

#[async_trait]
impl Action for BurnAll {
    fn name() -> &'static str {
        "burn-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        let mint_pubkey = Pubkey::from_str(&args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

        let _sig = burn(BurnArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_pubkey,
            priority: args.priority,
        })
        .await
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

        Ok(())
    }
}

pub async fn burn_all(args: BurnAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

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
    BurnAll::run(args).await?;

    Ok(())
}

pub async fn burn(args: BurnArgs) -> AnyResult<Signature> {
    let assoc = get_associated_token_address(&args.keypair.pubkey(), &args.mint_pubkey);
    let spl_token_program_id = spl_token::id();
    let metadata_pubkey = derive_metadata_pda(&args.mint_pubkey);
    let master_edition = derive_edition_pda(&args.mint_pubkey);

    let md_account = args.client.get_account_data(&metadata_pubkey)?;
    let metadata = Metadata::deserialize(&mut md_account.as_slice())?;

    // Is it a verified collection item?
    let collection_md = if let Some(collection) = metadata.collection {
        if collection.verified {
            let collection_metadata_pubkey = derive_metadata_pda(&collection.key);
            Some(collection_metadata_pubkey)
        } else {
            None
        }
    } else {
        None
    };

    let burn_ix = BurnNft {
        metadata: metadata_pubkey,
        owner: args.keypair.pubkey(),
        mint: args.mint_pubkey,
        token_account: assoc,
        master_edition_account: master_edition,
        spl_token_program: spl_token_program_id,
        collection_metadata: collection_md,
    }
    .instruction();

    let instructions = vec![burn_ix];

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&args.keypair.pubkey()),
        &[&*args.keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}

#[async_trait]
impl Action for BurnPrintAll {
    fn name() -> &'static str {
        "burn-print-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        let mint_pubkey = Pubkey::from_str(&args.mint_account)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;
        let master_mint_pubkey = Pubkey::from_str(&args.new_value)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

        let _sig = burn_print(BurnPrintArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint_pubkey,
            master_mint_pubkey,
            priority: args.priority,
        })
        .await
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

        Ok(())
    }
}

pub async fn burn_print_all(args: BurnPrintAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    // We don't support an optional payer for this action currently.
    let payer = None;

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::Single(args.master_mint),
        rate_limit: args.rate_limit,
        retries: args.retries,
        priority: args.priority,
    };
    BurnPrintAll::run(args).await?;

    Ok(())
}

pub async fn burn_print(args: BurnPrintArgs) -> AnyResult<Signature> {
    let print_edition_token =
        get_associated_token_address(&args.keypair.pubkey(), &args.mint_pubkey);

    // Find the master edition holder.
    let master_edition_owner =
        get_largest_token_account_owner(&args.client, args.master_mint_pubkey)?;
    let master_edition_token =
        get_associated_token_address(&master_edition_owner, &args.master_mint_pubkey);

    let spl_token_program_id = spl_token::id();
    let metadata_pubkey = derive_metadata_pda(&args.mint_pubkey);

    let master_edition_pda = derive_edition_pda(&args.master_mint_pubkey);
    let print_edition_pda = derive_edition_pda(&args.mint_pubkey);

    let data = args.client.get_account_data(&print_edition_pda)?;
    let print_edition = Edition::deserialize(&mut data.as_slice())?;

    let edition_marker_pda =
        derive_edition_marker_pda(&args.master_mint_pubkey, print_edition.edition);

    let burn_ix = BurnEditionNft {
        metadata: metadata_pubkey,
        owner: args.keypair.pubkey(),
        print_edition_mint: args.mint_pubkey,
        master_edition_mint: args.master_mint_pubkey,
        print_edition_token_account: print_edition_token,
        master_edition_token_account: master_edition_token,
        master_edition_account: master_edition_pda,
        print_edition_account: print_edition_pda,
        edition_marker_account: edition_marker_pda,
        spl_token_program: spl_token_program_id,
    }
    .instruction();

    let instructions = vec![burn_ix];

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&args.keypair.pubkey()),
        &[&*args.keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(sig)
}
