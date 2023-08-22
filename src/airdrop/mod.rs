use std::str::FromStr;

use anyhow::Result;
use dialoguer::Input;
use jib::{Jib, JibResult, Network};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::update::{parse_keypair, parse_mint_list, parse_solana_config};

pub struct AirdropSolArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub network: Network,
    pub recipient_list: Option<String>,
    pub cache_file: Option<String>,
    pub amount: u64,
}

pub async fn airdrop_sol(args: AirdropSolArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mut jib = Jib::new(vec![keypair], Network::Devnet)?;

    let mut instructions = vec![];

    let recipients = parse_mint_list(args.recipient_list, &args.cache_file)?.unwrap();

    let status_file_name = "statuses.json";

    for recipient in recipients {
        let recipient = Pubkey::from_str(&recipient).unwrap();
        instructions.push(solana_sdk::system_instruction::transfer(
            &jib.payer().pubkey(),
            &recipient,
            args.amount,
        ));
    }

    jib.set_instructions(instructions);
    let statuses = jib.hoist()?;

    if statuses.iter().any(|r| r.is_failure()) {
        let input: String = Input::new()
            .with_prompt("Some transactions failed. Retry the failed ones? (y/n)")
            .interact_text()
            .unwrap();

        if input.to_lowercase() == "y" {
            // Get transactions from statuses
            let mut transactions = vec![];
            for status in &statuses {
                if let JibResult::Failure(tx) = status {
                    transactions.push(tx.transaction.clone());
                }
            }
            let statuses = jib.submit_packed_transactions(transactions)?;

            if statuses.iter().any(|r| r.is_failure()) {
                println!("Some transactions failed. Writing to {status_file_name} and exiting.");
            }
        }
    }

    // Write statuses to file
    let statuses_file = std::fs::File::create(status_file_name)?;
    serde_json::to_writer_pretty(statuses_file, &statuses)?;

    Ok(())
}
