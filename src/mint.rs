use anyhow::{anyhow, Result};
use glob::glob;
use indicatif::ParallelProgressIterator;
use metaplex_token_metadata::instruction::{create_master_edition, create_metadata_accounts};
use rayon::prelude::*;
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
use crate::parse::parse_keypair;
use crate::{constants::*, parse::convert_local_to_remote_data};

const MINT_LAYOUT: u64 = 82;

pub fn mint_list(
    client: &RpcClient,
    keypair: String,
    receiver: Option<String>,
    list_dir: String,
    immutable: bool,
) -> Result<()> {
    let path = Path::new(&list_dir).join("*.json");
    let pattern = path.to_str().ok_or(anyhow!("Invalid directory path"))?;

    let (paths, errors): (Vec<_>, Vec<_>) = glob(pattern)?.into_iter().partition(Result::is_ok);

    let paths: Vec<_> = paths.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    paths.par_iter().progress().for_each(|path| {
        match mint_one(client, &keypair, &receiver, path, immutable) {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to mint {:?}: {}", &path, e),
        }
    });

    // TODO: handle errors in a better way and log instead of print.
    if !errors.is_empty() {
        eprintln!("Failed to read some of the files with the following errors:");
        for error in errors {
            eprintln!("{}", error);
        }
    }

    Ok(())
}

pub fn mint_one<P: AsRef<Path>>(
    client: &RpcClient,
    keypair: &String,
    receiver: &Option<String>,
    nft_data_file: P,
    immutable: bool,
) -> Result<()> {
    let keypair = parse_keypair(&keypair)?;

    let receiver = if let Some(address) = receiver {
        Pubkey::from_str(&address)?
    } else {
        keypair.pubkey()
    };

    let f = File::open(nft_data_file)?;
    let nft_data: NFTData = serde_json::from_reader(f)?;

    let (tx_id, mint_account) = mint(client, keypair, receiver, nft_data, immutable)?;
    println!("Tx id: {:?}\nMint account: {:?}", tx_id, mint_account);

    Ok(())
}

pub fn mint(
    client: &RpcClient,
    funder: Keypair,
    receiver: Pubkey,
    nft_data: NFTData,
    immutable: bool,
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

    let instructions = &[
        create_mint_account_ix,
        init_mint_ix,
        create_assoc_account_ix,
        mint_to_ix,
        create_metadata_account_ix,
        create_master_edition_account_ix,
    ];

    let (recent_blockhash, _) = client.get_recent_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&funder.pubkey()),
        &[&funder, &mint],
        recent_blockhash,
    );

    let tx_id = client.send_and_confirm_transaction(&tx)?;

    Ok((tx_id, mint.pubkey()))
}
