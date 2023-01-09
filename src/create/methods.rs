use std::path::PathBuf;

use anyhow::anyhow;
use metaboss_lib::derive::derive_edition_pda;
use mpl_token_metadata::instruction::create_master_edition_v3;
use solana_sdk::signature::read_keypair_file;

use super::*;

pub struct CreateMetadataArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint: String,
    pub metadata: String,
    pub immutable: bool,
}

pub fn create_metadata(args: CreateMetadataArgs) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(&args.mint)?;
    let metadata_pubkey = derive_metadata_pda(&mint_pubkey);

    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let f = File::open(args.metadata)?;
    let data: Data = serde_json::from_reader(f)?;

    let ix = create_metadata_accounts_v3(
        METADATA_PROGRAM_ID,
        metadata_pubkey,
        mint_pubkey,
        keypair.pubkey(),
        keypair.pubkey(),
        keypair.pubkey(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        true,
        !args.immutable,
        None,
        None,
        None,
    );

    let instructions = vec![ix];

    let sig = send_and_confirm_transaction(&args.client, keypair, &instructions)?;

    println!("Signature: {}", sig);

    Ok(())
}

pub struct CreateFungibleArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub metadata: String,
    pub decimals: u8,
    pub initial_supply: Option<f64>,
    pub immutable: bool,
}

#[derive(Deserialize)]
pub struct FungibleFields {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

pub fn create_fungible(args: CreateFungibleArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let f = File::open(args.metadata)?;
    let data: FungibleFields = serde_json::from_reader(f)?;

    let mint = Keypair::new();
    let metadata_pubkey = derive_metadata_pda(&mint.pubkey());

    let mut instructions = Vec::new();

    // Allocate memory for the account
    let min_rent = args
        .client
        .get_minimum_balance_for_rent_exemption(MINT_LAYOUT as usize)?;

    // Create mint account
    let create_mint_account_ix = create_account(
        &keypair.pubkey(),
        &mint.pubkey(),
        min_rent,
        MINT_LAYOUT,
        &TOKEN_PROGRAM_ID,
    );
    instructions.push(create_mint_account_ix);

    // Initalize mint ix
    let init_mint_ix = initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &keypair.pubkey(),
        Some(&keypair.pubkey()),
        args.decimals,
    )?;
    instructions.push(init_mint_ix);

    // Derive associated token account
    let assoc = get_associated_token_address(&keypair.pubkey(), &mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix =
        create_associated_token_account(&keypair.pubkey(), &keypair.pubkey(), &mint.pubkey());
    instructions.push(create_assoc_account_ix);

    if let Some(initial_supply) = args.initial_supply {
        // Convert float to native token units
        let supply = (initial_supply * 10_f64.powi(args.decimals as i32)) as u64;

        // Mint to instruction
        let mint_to_ix = mint_to(
            &TOKEN_PROGRAM_ID,
            &mint.pubkey(),
            &assoc,
            &keypair.pubkey(),
            &[],
            supply,
        )?;
        instructions.push(mint_to_ix);
    }

    let metadata_ix = create_metadata_accounts_v3(
        METADATA_PROGRAM_ID,
        metadata_pubkey,
        mint.pubkey(),
        keypair.pubkey(),
        keypair.pubkey(),
        keypair.pubkey(),
        data.name,
        data.symbol,
        data.uri,
        None, // Fungible does not have creators
        0,    // Fungible does not have royalties
        true,
        !args.immutable,
        None,
        None,
        None,
    );
    instructions.push(metadata_ix);

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[&keypair, &mint],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );

    let sig = res?;
    println!("Signature: {sig}");
    println!("Mint: {}", mint.pubkey());
    println!("Metadata: {}", metadata_pubkey);

    Ok(())
}

pub struct CreateMasterEditionArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_authority: Option<PathBuf>,
    pub mint: Pubkey,
    pub max_supply: i64,
}

pub fn create_master_edition(args: CreateMasterEditionArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint_authority = if let Some(mint_authority) = args.mint_authority {
        read_keypair_file(&mint_authority)
            .map_err(|e| anyhow!(format!("Failed to read mint authority keypair file: {e}")))?
    } else {
        Keypair::from_bytes(&keypair.to_bytes())
            .map_err(|e| anyhow!(format!("Failed to create mint authority keypair: {e}")))?
    };

    let mint_pubkey = args.mint;
    let metadata_pubkey = derive_metadata_pda(&mint_pubkey);
    let edition_pubkey = derive_edition_pda(&mint_pubkey);

    let max_supply = match args.max_supply {
        i64::MIN..=-2 => panic!("Max supply: must be greater than -1"),
        -1 => None,
        0.. => Some(args.max_supply as u64),
    };

    let ix = create_master_edition_v3(
        METADATA_PROGRAM_ID,
        edition_pubkey,
        mint_pubkey,
        keypair.pubkey(),
        mint_authority.pubkey(),
        metadata_pubkey,
        keypair.pubkey(),
        max_supply,
    );

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&keypair.pubkey()),
        &[&keypair, &mint_authority],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );

    let sig = res?;
    println!("Signature: {sig}");
    println!("Edition: {}", edition_pubkey);

    Ok(())
}
