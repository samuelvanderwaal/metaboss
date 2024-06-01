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
use solana_sdk::{
    commitment_config::CommitmentConfig, compute_budget::ComputeBudgetInstruction,
    signature::read_keypair_file, system_instruction::create_account,
};
use spl_associated_token_account::{
    get_associated_token_address, get_associated_token_address_with_program_id,
    instruction::create_associated_token_account,
};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token::instruction::mint_to;
use spl_token_2022::{
    extension::{
        cpi_guard::instruction::enable_cpi_guard,
        interest_bearing_mint::instruction::initialize as initialize_interest_bearing,
        memo_transfer::instruction::enable_required_transfer_memos,
        metadata_pointer::instruction::initialize as initialize_metadata_pointer,
        transfer_fee::instruction::initialize_transfer_fee_config,
        transfer_hook::instruction::initialize as initialize_transfer_hook,
        BaseStateWithExtensions, ExtensionType, StateWithExtensions,
    },
    instruction::{
        initialize_account, initialize_immutable_owner, initialize_mint2,
        initialize_mint_close_authority, initialize_non_transferable_mint,
        initialize_permanent_delegate, mint_to_checked as mint_22_to,
    },
    state::{Account, Mint},
    ID as TOKEN_22_PROGRAM_ID,
};
use spl_token_metadata_interface::{
    instruction::{initialize as initialize_metadata, update_field as add_additional_metadata},
    state::{Field, TokenMetadata},
};

use crate::utils::create_token_if_missing_instruction;

use super::*;

const DEFAULT_COMPUTE_UNITS: u64 = 200_000;

pub struct CreateMetadataArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint: String,
    pub metadata: String,
    pub immutable: bool,
    pub priority: Priority,
    pub full_compute: bool,
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

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let mut instructions = vec![];

    if !args.full_compute {
        // Only set the compute unit limit if we're not doing a full compute
        let compute_units = get_compute_units(&args.client, &[create_ix.clone()], &[&keypair])?
            .unwrap_or(DEFAULT_COMPUTE_UNITS);

        instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            compute_units as u32,
        ));
    }

    // Always set the compute unit price
    instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
        micro_lamports,
    ));
    instructions.push(create_ix);

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
    pub full_compute: bool,
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

pub struct CreateFungible22Args {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub extensions: String,
    pub mint_path: Option<String>,
    pub decimals: u8,
    pub initial_supply: Option<u64>,
    pub priority: Priority,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransferFeeConfig {
    pub transfer_fee_config_authority: Option<String>,
    pub withdraw_withheld_authority: Option<String>,
    pub fee_basis_points: u16,
    pub max_fee: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct InterestBearingConfig {
    pub rate_authority: Option<String>,
    pub rate: i16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TransferHookConfig {
    pub program_id: Option<String>,
    pub authority: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MetadataConfig {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub additional_metadata: Option<Vec<[String; 2]>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Fungible22Fields {
    pub close_authority: Option<String>,
    pub permanent_delegate: Option<String>,
    pub non_transferrable: Option<bool>,
    pub transfer_fee: Option<TransferFeeConfig>,
    pub interest_bearing: Option<InterestBearingConfig>,
    pub transfer_hook: Option<TransferHookConfig>,
    pub metadata: Option<MetadataConfig>,
}

pub struct CreateFungible22TokenArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub extensions: String,
    pub mint_address: String,
    pub priority: Priority,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Fungible22TokenFields {
    pub require_memo: Option<bool>,
    pub cpi_guard: Option<bool>,
}

pub struct CreateMasterEditionArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_authority: Option<PathBuf>,
    pub mint: Pubkey,
    pub max_supply: i64,
    pub priority: Priority,
}

pub fn parse_pubkey(pubkey_str: &str) -> Result<Pubkey> {
    match Pubkey::from_str(pubkey_str) {
        Ok(key) => Ok(key),
        Err(_) => Err(anyhow!("Invalid pubkey passed {}", pubkey_str)),
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

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let mut extra_instructions = vec![];

    if !args.full_compute {
        // Only set the compute unit limit if not using full compute
        let compute_units = get_compute_units(&args.client, &instructions, &signers)?
            .unwrap_or(DEFAULT_COMPUTE_UNITS);

        extra_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
            compute_units as u32,
        ));
    }

    // Always set the compute unit price
    extra_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
        micro_lamports,
    ));

    instructions.splice(0..0, extra_instructions);

    let sig = send_and_confirm_tx(&args.client, &signers, &instructions)?;

    println!("Signature: {sig}");
    println!("Mint: {}", mint.pubkey());
    println!("Metadata: {metadata_pubkey}");

    Ok(())
}

pub fn create_fungible_22(args: CreateFungible22Args) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint = if let Some(path) = args.mint_path {
        read_keypair_file(&path)
            .map_err(|e| anyhow!(format!("Failed to read mint keypair file: {e}")))?
    } else {
        Keypair::new()
    };

    let f = File::open(args.extensions)?;
    let extensions_data: Fungible22Fields = serde_json::from_reader(f)?;

    let is_close_authority = extensions_data.close_authority.is_some();
    let is_permanent_delegate = extensions_data.permanent_delegate.is_some();
    let is_non_transferrable = extensions_data.non_transferrable.is_some();
    let is_transfer_fee = extensions_data.transfer_fee.is_some();
    let is_interest_bearing = extensions_data.interest_bearing.is_some();
    let is_transfer_hook = extensions_data.transfer_hook.is_some();
    let is_metadata = extensions_data.metadata.is_some();

    let mut extension_types = vec![];

    // Validations
    if is_non_transferrable && is_transfer_fee {
        return Err(anyhow!(
            "Cannot have both NonTransferrable and Transfer Fee extensions"
        ));
    }

    if is_non_transferrable && is_transfer_hook {
        return Err(anyhow!(
            "Cannot have both NonTransferrable and Transfer Hook extensions"
        ));
    }

    // Adding extensions
    if is_close_authority {
        extension_types.push(ExtensionType::MintCloseAuthority);
    }

    if is_permanent_delegate {
        extension_types.push(ExtensionType::PermanentDelegate);
    }

    if is_non_transferrable {
        let non_transferrable_flag = extensions_data.non_transferrable.unwrap();
        if non_transferrable_flag {
            extension_types.push(ExtensionType::NonTransferable);
        }
    }

    if is_transfer_fee {
        extension_types.push(ExtensionType::TransferFeeConfig);
    }

    if is_interest_bearing {
        extension_types.push(ExtensionType::InterestBearingConfig);
    }

    if is_transfer_hook {
        extension_types.push(ExtensionType::TransferHook);
    }

    if is_metadata {
        extension_types.push(ExtensionType::MetadataPointer);
    }

    // Create mint account
    let mint_pubkey = mint.pubkey();
    let keypair_pubkey = keypair.pubkey();

    let mint_size = ExtensionType::try_calculate_account_len::<Mint>(&extension_types)?;
    let metadata_size = if let Some(metadata_config) = extensions_data.metadata.clone() {
        let additional_metdata: Vec<(String, String)> =
            if let Some(additional_metadata) = metadata_config.additional_metadata {
                let mut metadata_tuple: Vec<(String, String)> = vec![];
                for metadata_pair in additional_metadata {
                    metadata_tuple.push((metadata_pair[0].clone(), metadata_pair[1].clone()));
                }
                metadata_tuple
            } else {
                vec![]
            };

        let metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey::try_from(Some(keypair_pubkey))?,
            mint: mint_pubkey,
            name: metadata_config.name,
            symbol: metadata_config.symbol,
            uri: metadata_config.uri,
            additional_metadata: additional_metdata,
        };

        metadata.tlv_size_of()
    } else {
        Ok(0)
    }?;

    let mint_rent = args
        .client
        .get_minimum_balance_for_rent_exemption(mint_size + metadata_size)?;

    let mut instructions = vec![];

    let create_mint_account_ix = create_account(
        &keypair_pubkey,
        &mint_pubkey,
        mint_rent,
        u64::try_from(mint_size).unwrap(),
        &TOKEN_22_PROGRAM_ID,
    );
    instructions.push(create_mint_account_ix);

    // Initialize extensions
    if is_metadata {
        let init_metadata_pointer_ix = initialize_metadata_pointer(
            &TOKEN_22_PROGRAM_ID,
            &mint_pubkey,
            Some(keypair_pubkey),
            Some(mint_pubkey),
        )?;
        instructions.push(init_metadata_pointer_ix);
    }

    if let Some(close_authority) = extensions_data.close_authority {
        let close_authority = parse_pubkey(&close_authority)?;
        let init_close_authority_ix = initialize_mint_close_authority(
            &TOKEN_22_PROGRAM_ID,
            &mint_pubkey,
            Some(&close_authority),
        )?;
        instructions.push(init_close_authority_ix);
    }

    if let Some(non_transferrable) = extensions_data.non_transferrable {
        if non_transferrable {
            let init_non_transferrable_ix =
                initialize_non_transferable_mint(&TOKEN_22_PROGRAM_ID, &mint_pubkey)?;
            instructions.push(init_non_transferrable_ix);
        }
    }

    if let Some(TransferFeeConfig {
        fee_basis_points,
        max_fee,
        transfer_fee_config_authority,
        withdraw_withheld_authority,
    }) = extensions_data.transfer_fee
    {
        let t_auth = if let Some(config_auth) = transfer_fee_config_authority {
            Some(parse_pubkey(&config_auth)?)
        } else {
            None
        };

        let w_auth = if let Some(withdraw_auth) = withdraw_withheld_authority {
            Some(parse_pubkey(&withdraw_auth)?)
        } else {
            None
        };

        let transfer_fee_config_authority = t_auth.as_ref();
        let withdraw_withheld_authority = w_auth.as_ref();

        let init_transfer_fee_ix = initialize_transfer_fee_config(
            &TOKEN_22_PROGRAM_ID,
            &mint_pubkey,
            transfer_fee_config_authority,
            withdraw_withheld_authority,
            fee_basis_points,
            max_fee,
        )?;

        instructions.push(init_transfer_fee_ix);
    }

    if let Some(permanent_delegate) = extensions_data.permanent_delegate {
        let permanent_delegate = parse_pubkey(&permanent_delegate)?;
        let init_permanent_delegate_ix =
            initialize_permanent_delegate(&TOKEN_22_PROGRAM_ID, &mint_pubkey, &permanent_delegate)?;
        instructions.push(init_permanent_delegate_ix);
    }

    if let Some(InterestBearingConfig {
        rate,
        rate_authority,
    }) = extensions_data.interest_bearing
    {
        let r_auth = if let Some(rate_auth) = rate_authority {
            Some(parse_pubkey(&rate_auth)?)
        } else {
            None
        };

        let init_interest_bearing_ix =
            initialize_interest_bearing(&TOKEN_22_PROGRAM_ID, &mint_pubkey, r_auth, rate)?;
        instructions.push(init_interest_bearing_ix);
    }

    if let Some(TransferHookConfig {
        program_id,
        authority,
    }) = extensions_data.transfer_hook
    {
        let program_id = if let Some(program_id) = program_id {
            Some(parse_pubkey(&program_id)?)
        } else {
            None
        };

        let authority = if let Some(authority) = authority {
            Some(parse_pubkey(&authority)?)
        } else {
            None
        };

        let init_transfer_hook_ix =
            initialize_transfer_hook(&TOKEN_22_PROGRAM_ID, &mint_pubkey, authority, program_id)?;
        instructions.push(init_transfer_hook_ix);
    }

    // Initialize mint
    let initialize_mint_ix = initialize_mint2(
        &TOKEN_22_PROGRAM_ID,
        &mint_pubkey,
        &keypair_pubkey,
        Some(&keypair_pubkey),
        args.decimals,
    )?;
    instructions.push(initialize_mint_ix);

    // Initialize metadata
    if let Some(MetadataConfig {
        name,
        uri,
        symbol,
        additional_metadata,
    }) = extensions_data.metadata
    {
        let init_metadata_ix = initialize_metadata(
            &TOKEN_22_PROGRAM_ID,
            &mint_pubkey,
            &keypair_pubkey,
            &mint_pubkey,
            &keypair_pubkey,
            name,
            symbol,
            uri,
        );
        instructions.push(init_metadata_ix);

        if let Some(additional_metadata) = additional_metadata {
            for field_value_pair in additional_metadata {
                let add_additional_metadata_ix = add_additional_metadata(
                    &TOKEN_22_PROGRAM_ID,
                    &mint_pubkey,
                    &keypair_pubkey,
                    Field::Key(field_value_pair[0].clone()),
                    field_value_pair[1].clone(),
                );
                instructions.push(add_additional_metadata_ix);
            }
        }
    }

    // Minting
    if let Some(initial_supply) = args.initial_supply {
        let supply = initial_supply
            .checked_mul(10u64.checked_pow(args.decimals.into()).unwrap())
            .unwrap();

        if supply > 0 {
            // Derive associated token account
            let associated_token_account = get_associated_token_address_with_program_id(
                &keypair_pubkey,
                &mint_pubkey,
                &TOKEN_22_PROGRAM_ID,
            );

            // Create associated token account if needed
            let create_token_ix = create_associated_token_account(
                &keypair_pubkey,
                &keypair_pubkey,
                &mint_pubkey,
                &TOKEN_22_PROGRAM_ID,
            );
            instructions.push(create_token_ix);

            // Mint to instruction
            let mint_to_ix = mint_22_to(
                &TOKEN_22_PROGRAM_ID,
                &mint_pubkey,
                &associated_token_account,
                &keypair_pubkey,
                &[],
                supply,
                args.decimals,
            )?;
            instructions.push(mint_to_ix);
        }
    }

    let signers = vec![&keypair, &mint];

    // Priority fees
    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    // Always set the compute unit price
    let compute_units =
        get_compute_units(&args.client, &instructions, &signers)?.unwrap_or(DEFAULT_COMPUTE_UNITS);

    let extra_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(compute_units as u32),
        ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
    ];

    instructions.splice(0..0, extra_instructions);

    let sig = send_and_confirm_tx(&args.client, &signers, &instructions)?;

    println!("Signature: {sig}");
    println!("Mint: {}", mint.pubkey());

    Ok(())
}

pub fn create_fungible_22_token(args: CreateFungible22TokenArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint_pubkey = Pubkey::from_str(&args.mint_address)?;
    let keypair_pubkey = keypair.pubkey();

    let mint_account_info = args
        .client
        .get_account_with_commitment(&mint_pubkey, CommitmentConfig::confirmed())?
        .value;

    if mint_account_info.is_none() {
        return Err(anyhow!("Invalid Mint Address"));
    }

    let token = Keypair::new();
    let destination_token_pubkey = token.pubkey();

    let destination_token_pubkey_info = args
        .client
        .get_account_with_commitment(&destination_token_pubkey, CommitmentConfig::confirmed())?
        .value;

    let mut instructions = vec![];

    if destination_token_pubkey_info.is_none() {
        let f = File::open(args.extensions)?;
        let extensions_data: Fungible22TokenFields = serde_json::from_reader(f)?;

        let is_require_memo = extensions_data.require_memo.is_some();
        let is_cpi_guard = extensions_data.cpi_guard.is_some();

        let mut extension_types = vec![ExtensionType::ImmutableOwner];

        if is_require_memo && extensions_data.require_memo.unwrap() {
            extension_types.push(ExtensionType::MemoTransfer);
        }

        if is_cpi_guard && extensions_data.cpi_guard.unwrap() {
            extension_types.push(ExtensionType::CpiGuard);
        }

        let mint_account_info_data = mint_account_info.unwrap().data;
        let mint_account_data = StateWithExtensions::<Mint>::unpack(&mint_account_info_data)?;
        let mint_extensions = mint_account_data.get_extension_types()?;
        let mut required_extensions =
            ExtensionType::get_required_init_account_extensions(&mint_extensions);

        for extension_type in extension_types.into_iter() {
            if !required_extensions.contains(&extension_type) {
                required_extensions.push(extension_type);
            }
        }

        let account_size =
            ExtensionType::try_calculate_account_len::<Account>(&required_extensions)?;
        let account_rent = args
            .client
            .get_minimum_balance_for_rent_exemption(account_size)?;

        instructions.push(create_account(
            &keypair_pubkey,
            &destination_token_pubkey,
            account_rent,
            u64::try_from(account_size).unwrap(),
            &TOKEN_22_PROGRAM_ID,
        ));

        instructions.push(initialize_immutable_owner(
            &TOKEN_22_PROGRAM_ID,
            &destination_token_pubkey,
        )?);

        instructions.push(initialize_account(
            &TOKEN_22_PROGRAM_ID,
            &destination_token_pubkey,
            &mint_pubkey,
            &keypair_pubkey,
        )?);

        if is_cpi_guard {
            instructions.push(enable_cpi_guard(
                &TOKEN_22_PROGRAM_ID,
                &destination_token_pubkey,
                &keypair_pubkey,
                &[],
            )?);
        }

        if is_require_memo {
            instructions.push(enable_required_transfer_memos(
                &TOKEN_22_PROGRAM_ID,
                &destination_token_pubkey,
                &keypair_pubkey,
                &[],
            )?);
        }
    }

    let signers = vec![&keypair, &token];

    let compute_units =
        get_compute_units(&args.client, &instructions, &signers)?.unwrap_or(200_000);

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let mut final_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(compute_units as u32),
        ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
    ];
    final_instructions.extend(instructions);

    let recent_blockhash = args.client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &final_instructions,
        Some(&keypair_pubkey),
        &signers,
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || args.client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    println!(
        "Token: {:?} created successfully!",
        destination_token_pubkey.to_string()
    );

    println!("Created in tx: {:?}", &sig);

    Ok(())
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
