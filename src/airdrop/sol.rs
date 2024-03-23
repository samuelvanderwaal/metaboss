use indicatif::ProgressBar;
use jib::JibFailedTransaction;
use metaboss_lib::data::Priority;

use super::*;

pub struct AirdropSolArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub recipient_list: Option<String>,
    pub cache_file: Option<String>,
    pub priority: Priority,
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

    let mut cache_file_name = format!("mb-cache-airdrop-{timestamp}.bin");
    let successful_tx_file_name = format!("mb-successful-airdrops-{timestamp}.json");

    let priority_fee = match args.priority {
        Priority::None => 200,         // 1 lamport
        Priority::Low => 200_000,      // 1_000 lamports
        Priority::Medium => 1_000_000, // 5_000 lamports
        Priority::High => 5_000_000,   // 25_000 lamports
        Priority::Max => 20_000_000,   // 100_000 lamports
    };

    jib.set_priority_fee(priority_fee);
    jib.set_compute_budget(AIRDROP_SOL_CU);

    if let Some(rate) = args.rate_limit {
        jib.set_rate_limit(rate);
    }

    // Airdrop case
    let results = if let Some(list_file) = args.recipient_list {
        let airdrop_list: HashMap<String, u64> = serde_json::from_reader(File::open(list_file)?)?;

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
        println!("Airdropping to {} recipients...", airdrop_list.len());

        jib.set_instructions(instructions);
        jib.hoist().await?

    // Retry case
    } else if let Some(cache_file) = args.cache_file {
        cache_file_name = PathBuf::from(cache_file.clone())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let failed_txes: Vec<JibFailedTransaction> =
            bincode::deserialize_from(File::open(cache_file)?)?;
        jib.retry_failed(failed_txes).await?
    } else {
        eprintln!("No recipient list or cache file provided.");
        std::process::exit(1);
    };

    if results.iter().any(|r| r.is_failure()) {
        println!(
            "Some transactions failed. Check the cache file for details by running `metaboss airdrop read-cache {cache_file_name}` to convert it to a JSON file.");
    }

    let mut successes = vec![];
    let mut failures = vec![];

    results.into_iter().for_each(|r| {
        if r.is_failure() {
            let failure = r.get_failure().unwrap();
            failures.push(failure);
        } else {
            debug!("Transaction successful: {}", r.signature().unwrap()); // Signatures exist on successes.
            successes.push(r.signature().unwrap()); // Signatures exist on successes.
        }
    });

    // Write cache file and successful transactions.
    if !successes.is_empty() {
        let successful_tx_file = std::fs::File::create(successful_tx_file_name)?;
        serde_json::to_writer_pretty(successful_tx_file, &successes)?;
    }

    let pb = ProgressBar::new_spinner();
    pb.set_message("Writing cache file...");
    pb.enable_steady_tick(100);

    let cache_file = std::fs::File::create(cache_file_name)?;
    bincode::serialize_into(cache_file, &failures)?;
    pb.finish_and_clear();

    Ok(())
}
