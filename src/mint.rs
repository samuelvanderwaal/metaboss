use anyhow::Result;
use metaplex_token_metadata::instruction::{create_master_edition, create_metadata_accounts};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    ID as TOKEN_PROGRAM_ID,
};
use std::{fs::File, str::FromStr};

use crate::data::NFTData;
use crate::parse::parse_keypair;
use crate::{constants::*, parse::convert_local_to_remote_data};

const MINT_LAYOUT: u64 = 82;

pub fn mint_nft(client: &RpcClient, keypair: &String, json_file: &String) -> Result<()> {
    let keypair = parse_keypair(keypair)?;
    let metaplex_program_id = Pubkey::from_str(METAPLEX_PROGRAM_ID)?;
    let mint = Keypair::new();

    let f = File::open(json_file)?;
    let json_data: NFTData = serde_json::from_reader(f)?;

    // Convert local NFTData type to Metaplex Data type
    let data = convert_local_to_remote_data(json_data)?;

    // Allocate memory for the account
    let min_rent = client.get_minimum_balance_for_rent_exemption(MINT_LAYOUT as usize)?;

    // Create mint account
    let create_mint_account_ix = create_account(
        &keypair.pubkey(),
        &mint.pubkey(),
        min_rent,
        MINT_LAYOUT,
        &TOKEN_PROGRAM_ID,
    );

    // Initalize mint ix
    let init_mint_ix = initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &keypair.pubkey(),
        Some(&keypair.pubkey()),
        0,
    )?;

    // Derive associated token account
    let assoc = get_associated_token_address(&keypair.pubkey(), &mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix =
        create_associated_token_account(&keypair.pubkey(), &keypair.pubkey(), &mint.pubkey());

    // Mint to instruction
    let mint_to_ix = mint_to(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &assoc,
        &keypair.pubkey(),
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
        keypair.pubkey(),
        keypair.pubkey(),
        keypair.pubkey(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        true,
        true,
    );

    let create_master_edition_account_ix = create_master_edition(
        metaplex_program_id,
        master_edition_account,
        mint.pubkey(),
        keypair.pubkey(),
        keypair.pubkey(),
        metadata_account,
        keypair.pubkey(),
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
        Some(&keypair.pubkey()),
        &[&keypair, &mint],
        recent_blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx)?;
    println!("Tx sig: {:?}", sig);

    Ok(())
}
