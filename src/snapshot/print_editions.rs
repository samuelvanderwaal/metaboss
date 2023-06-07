use indexmap::IndexMap;
use metaboss_lib::{
    decode::decode_metadata_from_mint, derive::derive_edition_pda,
    snapshot::get_metadata_accounts_by_creator,
};
use mpl_token_metadata::state::{Edition, TokenMetadataAccount};

use crate::spinner::create_spinner;

use super::*;

// Process
// Decode master edition NFT metadata account
// Find first verified creator
// Snapshot all metadata accounts with the same first verified creator
// Decode the metadata accounts
// Get the mint from the metadata accounts
// Derive the edition for each
// Decode the edition and check if it's a master or a print
// If it's a print and it's parent is the master edition, add it to the list

pub struct SnapshotPrintEditionsArgs {
    pub client: RpcClient,
    pub master_mint: String,
    pub creator: Option<String>,
    pub output: String,
}

pub async fn snapshot_print_editions<'a>(args: SnapshotPrintEditionsArgs) -> Result<()> {
    let master_mint_pubkey = Pubkey::from_str(&args.master_mint)?;
    let master_edition_pubkey = derive_edition_pda(&master_mint_pubkey);

    let first_verified_creator = if let Some(creator) = args.creator {
        creator
    } else {
        let master_nft = decode_metadata_from_mint(&args.client, args.master_mint.clone())?;

        master_nft
            .data
            .creators
            .ok_or(anyhow!("No creators found"))?
            .iter()
            .find(|c| c.verified)
            .ok_or_else(|| anyhow!("No verified creators found"))?
            .address
            .to_string()
    };

    let spinner = create_spinner("Fetching metadata accounts...");
    let accounts = get_metadata_accounts_by_creator(&args.client, &first_verified_creator, 0)?;
    spinner.finish();

    let spinner = create_spinner("Converting to mints...");
    let mints = accounts
        .into_iter()
        .map(|(_, mint)| mint)
        .map(|a| Metadata::safe_deserialize(a.data.as_slice()).unwrap())
        .map(|m| m.mint)
        .collect::<Vec<_>>();
    spinner.finish();

    let mut edition_mints = IndexMap::new();

    let spinner = create_spinner("Finding edition mints...");
    for m in mints {
        let edition = derive_edition_pda(&m);
        let edition_account = &args.client.get_account(&edition)?;
        let edition = match Edition::safe_deserialize(edition_account.data.as_slice()) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if edition.parent == master_edition_pubkey {
            edition_mints.insert(edition.edition, m.to_string());
        }
    }
    spinner.finish();

    edition_mints.sort_keys();

    println!("Found {} editions", edition_mints.len());

    println!("Writing to file...");
    let mut file = File::create(format!(
        "{}/{}_mint_accounts.json",
        args.output, &args.master_mint
    ))?;
    serde_json::to_writer_pretty(&mut file, &edition_mints)?;

    Ok(())
}
