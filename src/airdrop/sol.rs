use super::*;

pub struct AirdropSolArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub network: Network,
    pub recipient_list: Option<String>,
    pub cache_file: Option<String>,
    pub boost: bool,
    pub rate_limit: Option<u64>,
}

pub async fn airdrop_sol(args: AirdropSolArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mut jib = Jib::new(vec![keypair], args.client.url())?;

    let mut instructions = vec![];

    if args.recipient_list.is_some() && args.cache_file.is_some() {
        eprintln!("Cannot provide both a recipient list and a cache file.");
        std::process::exit(1);
    }

    // Get the current time as yyyy-mm-dd-hh-mm-ss
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d-%H-%M-%S").to_string();

    let mut cache_file_name = format!("mb-cache-airdrop-{timestamp}.json");
    let successful_tx_file_name = format!("mb-successful-airdrops-{timestamp}.json");

    let mut airdrop_list: HashMap<String, u64> = if let Some(list_file) = args.recipient_list {
        serde_json::from_reader(File::open(list_file)?)?
    } else if let Some(cache_file) = args.cache_file {
        cache_file_name = PathBuf::from(cache_file.clone())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let failed_txes: Vec<FailedTransaction> = serde_json::from_reader(File::open(cache_file)?)?;
        failed_txes
            .iter()
            .flat_map(|f| f.recipients.clone())
            .collect()
    } else {
        eprintln!("No recipient list or cache file provided.");
        std::process::exit(1);
    };

    for (address, amount) in &airdrop_list {
        let pubkey = match Pubkey::from_str(address) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                eprintln!("Invalid address: {}, skipping...", address);
                continue;
            }
        };

        instructions.push(solana_sdk::system_instruction::transfer(
            &jib.payer().pubkey(),
            &pubkey,
            *amount,
        ));
    }

    if args.boost {
        jib.set_priority_fee(PRIORITY_FEE);
    }

    if let Some(rate) = args.rate_limit {
        jib.set_rate_limit(rate);
    }

    jib.set_instructions(instructions);
    let results = jib.hoist().await?;

    if results.iter().any(|r| r.is_failure()) {
        println!("Some transactions failed. Check the {cache_file_name} cache file for details.");
    }

    let mut successes = vec![];
    let mut failures = vec![];

    results.iter().for_each(|r| {
        if r.is_failure() {
            let account_keys = r.message().unwrap().account_keys; // Transactions exist on failures.
            let transaction_accounts = account_keys.iter().map(|k| k.to_string()).collect();

            // All accounts except the first and last are recipients.
            let recipients: HashMap<String, u64> = account_keys[1..account_keys.len() - 1]
                .iter()
                .map(|pubkey| pubkey.to_string())
                .map(|a| airdrop_list.remove_entry(&a).expect("Recipient not found"))
                .collect();

            failures.push(FailedTransaction {
                transaction_accounts,
                recipients,
                error: r.error().unwrap(), // Errors exist on failures.
            })
        } else {
            debug!("Transaction successful: {}", r.signature().unwrap()); // Signatures exist on successes.
            successes.push(r.signature().unwrap()); // Signatures exist on successes.
        }
    });

    // Write cache file and successful transactions.
    let successful_tx_file = std::fs::File::create(successful_tx_file_name)?;
    serde_json::to_writer_pretty(successful_tx_file, &successes)?;

    let cache_file = std::fs::File::create(cache_file_name)?;
    serde_json::to_writer_pretty(cache_file, &failures)?;

    Ok(())
}
