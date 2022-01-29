use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
        signature::keypair::Keypair,
    },
    Client, Cluster,
};
use anyhow::Result;
use std::str::FromStr;

use mpl_candy_machine::accounts as nft_accounts;
use mpl_candy_machine::instruction as nft_instruction;

use crate::parse::parse_keypair;

pub struct WithdrawArgs {
    pub rpc_url: String,
    pub candy_machine_id: String,
    pub keypair: String,
}

pub fn withdraw(args: WithdrawArgs) -> Result<()> {
    let opts = CommitmentConfig::confirmed();
    let rpc_url = args.rpc_url.clone();
    let ws_url = rpc_url.replace("http", "ws");
    let cluster = Cluster::Custom(rpc_url, ws_url);

    let keypair = parse_keypair(&args.keypair)?;
    let key_bytes = keypair.to_bytes();
    let payer = Keypair::from_bytes(&key_bytes)?;

    let anchor_client = Client::new_with_options(cluster, payer, opts);

    let pid = "cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ"
        .parse()
        .expect("Failed to parse PID");
    let program = anchor_client.program(pid);
    let payer = program.payer();

    let candy_machine = Pubkey::from_str(&args.candy_machine_id)?;

    let account = match program.rpc().get_account(&candy_machine) {
        Ok(account) => account,
        Err(_) => {
            println!("No such account exists.");
            return Ok(());
        }
    };

    let sol = account.lamports as f64 / LAMPORTS_PER_SOL as f64;

    println!(
        "Withdrawing {sol} SOL from candy machine {}",
        &candy_machine
    );
    let sig = program
        .request()
        .accounts(nft_accounts::WithdrawFunds {
            candy_machine,
            authority: payer,
        })
        .args(nft_instruction::WithdrawFunds {})
        .send()?;

    println!("Transaction submitted with id of: {}", sig);

    Ok(())
}
