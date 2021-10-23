use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;

use metaboss::decode::{decode_metadata, decode_metadata_all};
use metaboss::mint::{mint_list, mint_one};
use metaboss::opt::*;
use metaboss::parse::parse_solana_config;
use metaboss::sign::sign;
use metaboss::snapshot::{get_cm_accounts, get_mints, get_snapshot};
use metaboss::update_metadata::*;

fn main() -> Result<()> {
    let sol_config = parse_solana_config();

    let (mut rpc, commitment) = if let Some(config) = sol_config {
        (config.json_rpc_url, config.commitment)
    } else {
        (
            "https://api.devnet.solana.com".to_string(),
            "confirmed".to_string(),
        )
    };

    let options = Opt::from_args();

    if let Some(cli_rpc) = options.rpc {
        rpc = cli_rpc.clone();
    }
    let commitment = CommitmentConfig::from_str(&commitment)?;

    let timeout = Duration::from_secs(60);

    let client = RpcClient::new_with_timeout_and_commitment(rpc, timeout, commitment);
    match options.cmd {
        Command::Decode { decode_subcommands } => process_decode(&client, decode_subcommands)?,
        Command::Mint { mint_subcommands } => process_mint(&client, mint_subcommands)?,
    }

    Ok(())
}

fn process_decode(client: &RpcClient, commands: DecodeSubcommands) -> Result<()> {
    match commands {
        DecodeSubcommands::Mint {
            account,
            list_path,
            ref output,
        } => decode_metadata(client, account.as_ref(), list_path.as_ref(), output)?,
    }
    Ok(())
}

fn process_mint(client: &RpcClient, commands: MintSubcommands) -> Result<()> {
    match commands {
        MintSubcommands::One {
            keypair,
            receiver,
            nft_data_file,
        } => mint_one(&client, &keypair, &receiver, nft_data_file),
        MintSubcommands::List {
            keypair,
            receiver,
            nft_data_dir,
        } => mint_list(&client, keypair, receiver, nft_data_dir),
    }
}
//     Command::Decode {
//         ref mint_account,
//         ref output,
//     } => decode_metadata(&client, mint_account, output)?,
// Command::DecodeAll {
//     ref json_file,
//     ref output,
// } => decode_metadata_all(&client, json_file, output)?,
// Command::GetMints {
//     ref update_authority,
//     ref candy_machine_id,
//     ref output,
// } => get_mints(&client, update_authority, candy_machine_id, output)?,
// Command::GetCMAccounts {
//     ref update_authority,
//     ref output,
// } => get_cm_accounts(&client, update_authority, output)?,
// Command::MintNFT {
//     ref keypair,
//     ref json_file,
// } => mint_nft(&client, keypair, json_file)?,
// Command::UpdateNFT {
//     ref keypair,
//     ref mint_account,
//     ref json_file,
// } => update_nft(&client, keypair, mint_account, json_file)?,
// Command::SetNewURI {
//     ref keypair,
//     ref mint_account,
//     ref new_uri,
// } => set_new_uri(&client, keypair, mint_account, new_uri)?,
// Command::SetUpdateAuthority {
//     ref keypair,
//     ref mint_account,
//     ref new_update_authority,
// } => set_update_authority(&client, keypair, mint_account, new_update_authority)?,
// Command::SetPrimarySaleHappened {
//     ref keypair,
//     ref mint_account,
// } => set_primary_sale_happened(&client, keypair, mint_account)?,
// Command::SetUpdateAuthorityAll {
//     ref keypair,
//     ref json_file,
// } => set_update_authority_all(&client, keypair, json_file)?,
// Command::Sign {
//     ref keypair,
//     ref candy_machine_id,
// } => sign(&client, keypair, candy_machine_id)?,
// Command::Snapshot {
//     ref update_authority,
//     ref candy_machine_id,
//     ref output,
// } => get_snapshot(&client, update_authority, candy_machine_id, output)?,
// }
