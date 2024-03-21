use std::path::PathBuf;

use anyhow::anyhow;
use metaboss_lib::{
    data::Priority,
    derive::derive_edition_pda,
    transaction::{get_compute_units, send_and_confirm_tx},
};
use mpl_token_metadata::{
    instructions::{CreateBuilder, CreateMasterEditionV3Builder},
    types::{CreateArgs, DataV2, TokenStandard},
};
use solana_sdk::{compute_budget::ComputeBudgetInstruction, signature::read_keypair_file};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::mint_to;

use crate::utils::create_token_if_missing_instruction;

use super::*;

// Arbitrary and capricious. Only used if the tx simulation does not return a value.
const DEFAULT_COMPUTE_UNITS: u64 = 150_000;

pub struct CreateMetadataArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint: String,
    pub metadata: String,
    pub immutable: bool,
    pub priority: Priority,
}

pub fn create_metadata(args: CreateMetadataArgs) -> Result<()> {
    let mint_pubkey = Pubkey::from_str(&args.mint)?;
    let metadata_pubkey = derive_metadata_pda(&mint_pubkey);

    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let f = File::open(args.metadata)?;
    let data: FungibleFields = serde_json::from_reader(f)?;

    let data_v2 = DataV2 {
        name: data.name,
        symbol: data.symbol,
        uri: data.uri,
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let create_args = CreateArgs::V1 {
        name: data_v2.name,
        symbol: data_v2.symbol,
        uri: data_v2.uri,
        seller_fee_basis_points: data_v2.seller_fee_basis_points,
        creators: data_v2.creators,
        primary_sale_happened: false,
        is_mutable: !args.immutable,
        token_standard: TokenStandard::Fungible,
        collection: None,
        uses: None,
        collection_details: None,
        decimals: None,
        rule_set: None,
        print_supply: None,
    };

    let create_ix = CreateBuilder::new()
        .metadata(metadata_pubkey)
        .mint(mint_pubkey, false)
        .authority(keypair.pubkey())
        .payer(keypair.pubkey())
        .update_authority(keypair.pubkey(), true)
        .create_args(create_args)
        .instruction();

    let compute_units = get_compute_units(&args.client, &[create_ix.clone()], &[&keypair])?
        .unwrap_or(DEFAULT_COMPUTE_UNITS);

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(compute_units as u32),
        ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
        create_ix,
    ];

    let sig = send_and_confirm_transaction(&args.client, keypair, &instructions)?;

    println!("Signature: {sig}");

    Ok(())
}

pub struct CreateFungibleArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub metadata: String,
    pub mint_path: Option<String>,
    pub decimals: u8,
    pub initial_supply: Option<f64>,
    pub immutable: bool,
    pub priority: Priority,
}

#[derive(Deserialize)]
pub struct FungibleFields {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl From<FungibleFields> for DataV2 {
    fn from(value: FungibleFields) -> Self {
        DataV2 {
            name: value.name,
            symbol: value.symbol,
            uri: value.uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        }
    }
}

pub fn create_fungible(args: CreateFungibleArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint = if let Some(path) = args.mint_path {
        read_keypair_file(&path)
            .map_err(|e| anyhow!(format!("Failed to read mint keypair file: {e}")))?
    } else {
        Keypair::new()
    };

    let metadata_pubkey = derive_metadata_pda(&mint.pubkey());

    let f = File::open(args.metadata)?;
    let data: FungibleFields = serde_json::from_reader(f)?;

    let create_args = CreateArgs::V1 {
        name: data.name,
        symbol: data.symbol,
        uri: data.uri,
        seller_fee_basis_points: 0,
        creators: None,
        primary_sale_happened: false,
        is_mutable: !args.immutable,
        token_standard: TokenStandard::Fungible,
        collection: None,
        uses: None,
        collection_details: None,
        decimals: Some(args.decimals),
        rule_set: None,
        print_supply: None,
    };

    let create_ix = CreateBuilder::new()
        .metadata(metadata_pubkey)
        .mint(mint.pubkey(), true)
        .authority(keypair.pubkey())
        .payer(keypair.pubkey())
        .update_authority(keypair.pubkey(), true)
        .create_args(create_args)
        .instruction();

    let mut instructions = vec![create_ix];

    if let Some(initial_supply) = args.initial_supply {
        // Convert float to native token units
        let supply = (initial_supply * 10_f64.powi(args.decimals as i32)) as u64;

        // Derive associated token account
        let assoc = get_associated_token_address(&keypair.pubkey(), &mint.pubkey());

        // Create associated token account if needed
        instructions.push(create_token_if_missing_instruction(
            &keypair.pubkey(),
            &assoc,
            &mint.pubkey(),
            &keypair.pubkey(),
            &assoc,
        ));

        // Mint to instruction
        let mint_to_ix = mint_to(
            &spl_token::ID,
            &mint.pubkey(),
            &assoc,
            &keypair.pubkey(),
            &[],
            supply,
        )?;
        instructions.push(mint_to_ix);
    }

    let signers = vec![&keypair, &mint];

    let compute_units =
        get_compute_units(&args.client, &instructions, &signers)?.unwrap_or(DEFAULT_COMPUTE_UNITS);

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    instructions.splice(
        0..0,
        vec![
            ComputeBudgetInstruction::set_compute_unit_limit(compute_units as u32),
            ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
        ],
    );

    println!("Instructions: {}", instructions.len());

    let sig = send_and_confirm_tx(&args.client, &signers, &instructions)?;

    println!("Signature: {sig}");
    println!("Mint: {}", mint.pubkey());
    println!("Metadata: {metadata_pubkey}");

    Ok(())
}

pub struct CreateMasterEditionArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_authority: Option<PathBuf>,
    pub mint: Pubkey,
    pub max_supply: i64,
    pub priority: Priority,
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

    let mut builder = CreateMasterEditionV3Builder::new();
    builder
        .edition(edition_pubkey)
        .mint(mint_pubkey)
        .update_authority(keypair.pubkey())
        .mint_authority(mint_authority.pubkey())
        .metadata(metadata_pubkey)
        .payer(keypair.pubkey());

    if let Some(max_supply) = max_supply {
        builder.max_supply(max_supply);
    }
    let ix = builder.instruction();

    let signers = vec![&keypair, &mint_authority];

    let compute_units =
        get_compute_units(&args.client, &[ix.clone()], &signers)?.unwrap_or(DEFAULT_COMPUTE_UNITS);

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(compute_units as u32),
        ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
        ix,
    ];

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &signers,
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );

    let sig = res?;
    println!("Signature: {sig}");
    println!("Edition: {edition_pubkey}");

    Ok(())
}
