use anyhow::{anyhow, Result};
use glob::glob;
use log::{error, info};
use mpl_token_metadata::instruction::{
    create_master_edition, create_metadata_accounts, update_metadata_accounts,
};
use rayon::prelude::*;
use reqwest;
use retry::{delay::Exponential, retry};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    ID as TOKEN_PROGRAM_ID,
};
use std::{fs::File, path::Path, str::FromStr};

use crate::data::NFTData;
use crate::limiter::create_rate_limiter;
use crate::parse::*;
use crate::sign::sign_one;
use crate::{constants::*, parse::convert_local_to_remote_data};

const MINT_LAYOUT: u64 = 82;

pub fn mint_list(
    client: &RpcClient,
    keypair_path: Option<String>,
    receiver: Option<String>,
    list_dir: Option<String>,
    external_metadata_uris: Option<String>,
    immutable: bool,
    primary_sale_happened: bool,
    sign: bool,
) -> Result<()> {
    if !is_only_one_option(&list_dir, &external_metadata_uris) {
        return Err(anyhow!(
            "Only one of --list-dir or --external-metadata-uris can be specified"
        ));
    }

    if let Some(list_dir) = list_dir {
        mint_from_files(
            client,
            keypair_path,
            receiver,
            list_dir,
            immutable,
            primary_sale_happened,
            sign,
        )?;
    } else if let Some(external_metadata_uris) = external_metadata_uris {
        mint_from_uris(
            client,
            keypair_path,
            receiver,
            external_metadata_uris,
            immutable,
            primary_sale_happened,
            sign,
        )?;
    } else {
        return Err(anyhow!(
            "Either --list-dir or --external-metadata-uris must be specified"
        ));
    }

    Ok(())
}

pub fn mint_from_files(
    client: &RpcClient,
    keypair_path: Option<String>,
    receiver: Option<String>,
    list_dir: String,
    immutable: bool,
    primary_sale_happened: bool,
    sign: bool,
) -> Result<()> {
    let use_rate_limit = *USE_RATE_LIMIT.read().unwrap();
    let handle = create_rate_limiter();

    let path = Path::new(&list_dir).join("*.json");
    let pattern = path.to_str().ok_or(anyhow!("Invalid directory path"))?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    paths.par_iter().for_each(|path| {
        let mut handle = handle.clone();
        if use_rate_limit {
            handle.wait();
        }

        match mint_one(
            client,
            keypair_path.clone(),
            &receiver,
            Some(path),
            None,
            immutable,
            primary_sale_happened,
            sign,
        ) {
            Ok(_) => (),
            Err(e) => error!("Failed to mint {:?}: {}", &path, e),
        }
    });

    // TODO: handle errors in a better way.
    if !errors.is_empty() {
        error!("Failed to read some of the files with the following errors:");
        for error in errors {
            error!("{}", error);
        }
    }

    Ok(())
}

pub fn mint_from_uris(
    client: &RpcClient,
    keypair_path: Option<String>,
    receiver: Option<String>,
    external_metadata_uris_path: String,
    immutable: bool,
    primary_sale_happened: bool,
    sign: bool,
) -> Result<()> {
    let f = File::open(external_metadata_uris_path)?;
    let external_metadata_uris: Vec<String> = serde_json::from_reader(f)?;

    external_metadata_uris
        .par_iter()
        // .progress()
        .for_each(|uri| {
            match mint_one(
                client,
                keypair_path.clone(),
                &receiver,
                None::<String>,
                Some(uri),
                immutable,
                primary_sale_happened,
                sign,
            ) {
                Ok(_) => (),
                Err(e) => error!("Failed to mint {:?}: {}", &uri, e),
            }
        });

    Ok(())
}
pub fn mint_one<P: AsRef<Path>>(
    client: &RpcClient,
    keypair_path: Option<String>,
    receiver: &Option<String>,
    nft_data_file: Option<P>,
    external_metadata_uri: Option<&String>,
    immutable: bool,
    primary_sale_happened: bool,
    sign: bool,
) -> Result<()> {
    if !is_only_one_option(&nft_data_file, &external_metadata_uri) {
        return Err(anyhow!(
            "You must supply either --nft_data_file or --external-metadata-uris but not both"
        ));
    }

    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path.clone(), solana_opts);

    let receiver = if let Some(address) = receiver {
        Pubkey::from_str(&address)?
    } else {
        keypair.pubkey()
    };

    let nft_data: NFTData = if let Some(nft_data_file) = nft_data_file {
        let f = File::open(nft_data_file)?;
        serde_json::from_reader(f)?
    } else if let Some(external_metadata_uri) = external_metadata_uri {
        let body: Value = reqwest::blocking::get(external_metadata_uri)?.json()?;
        let creators_json = body
            .get("properties")
            .ok_or_else(|| anyhow!("Bad JSON"))?
            .get("creators")
            .ok_or_else(|| anyhow!("Bad JSON"))?;
        let name = parse_name(&body)?;
        let creators = parse_creators(&creators_json)?;
        let symbol = parse_symbol(&body)?;
        let seller_fee_basis_points = parse_seller_fee_basis_points(&body)?;
        NFTData {
            name,
            symbol,
            creators: Some(creators),
            uri: external_metadata_uri.to_string(),
            seller_fee_basis_points,
        }
    } else {
        return Err(anyhow!(
            "You must supply either --nft_data_file or --external-metadata-uris but not both"
        ));
    };

    let (tx_id, mint_account) = mint(
        client,
        keypair,
        receiver,
        nft_data,
        immutable,
        primary_sale_happened,
    )?;
    info!("Tx id: {:?}\nMint account: {:?}", &tx_id, &mint_account);
    let message = format!("Tx id: {:?}\nMint account: {:?}", &tx_id, &mint_account,);
    println!("{}", message);
    if sign {
        //TODO: Error handling
        sign_one(client, keypair_path.clone(), mint_account.to_string())?;
    }

    Ok(())
}

pub fn mint(
    client: &RpcClient,
    funder: Keypair,
    receiver: Pubkey,
    nft_data: NFTData,
    immutable: bool,
    primary_sale_happened: bool,
) -> Result<(Signature, Pubkey)> {
    let metaplex_program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint = Keypair::new();

    // Convert local NFTData type to Metaplex Data type
    let data = convert_local_to_remote_data(nft_data)?;

    // Allocate memory for the account
    let min_rent = client.get_minimum_balance_for_rent_exemption(MINT_LAYOUT as usize)?;

    // Create mint account
    let create_mint_account_ix = create_account(
        &funder.pubkey(),
        &mint.pubkey(),
        min_rent,
        MINT_LAYOUT,
        &TOKEN_PROGRAM_ID,
    );

    // Initalize mint ix
    let init_mint_ix = initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &funder.pubkey(),
        Some(&funder.pubkey()),
        0,
    )?;

    // Derive associated token account
    let assoc = get_associated_token_address(&receiver, &mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix =
        create_associated_token_account(&funder.pubkey(), &receiver, &mint.pubkey());

    // Mint to instruction
    let mint_to_ix = mint_to(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &assoc,
        &funder.pubkey(),
        &[],
        1,
    )?;

    // Derive metadata account
    let metadata_seeds = &[
        "metadata".as_bytes(),
        &metaplex_program_id.to_bytes(),
        &mint.pubkey().to_bytes(),
    ];
    let (metadata_account, _pda) =
        Pubkey::find_program_address(metadata_seeds, &metaplex_program_id);

    // Derive Master Edition account
    let master_edition_seeds = &[
        "metadata".as_bytes(),
        &metaplex_program_id.to_bytes(),
        &mint.pubkey().to_bytes(),
        "edition".as_bytes(),
    ];
    let (master_edition_account, _pda) =
        Pubkey::find_program_address(master_edition_seeds, &metaplex_program_id);

    let create_metadata_account_ix = create_metadata_accounts(
        metaplex_program_id,
        metadata_account,
        mint.pubkey(),
        funder.pubkey(),
        funder.pubkey(),
        funder.pubkey(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        true,
        !immutable,
    );

    let create_master_edition_account_ix = create_master_edition(
        metaplex_program_id,
        master_edition_account,
        mint.pubkey(),
        funder.pubkey(),
        funder.pubkey(),
        metadata_account,
        funder.pubkey(),
        Some(0),
    );

    let mut instructions = vec![
        create_mint_account_ix,
        init_mint_ix,
        create_assoc_account_ix,
        mint_to_ix,
        create_metadata_account_ix,
        create_master_edition_account_ix,
    ];

    if primary_sale_happened {
        let ix = update_metadata_accounts(
            metaplex_program_id,
            metadata_account,
            funder.pubkey(),
            None,
            None,
            Some(true),
        );
        instructions.push(ix);
    }

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&funder.pubkey()),
        &[&funder, &mint],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok((sig, mint.pubkey()))
}
