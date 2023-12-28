use borsh::{BorshDeserialize, BorshSerialize};
use metaboss_lib::transaction::send_and_confirm_tx;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_pack::Pack,
    pubkey, system_program,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer_checked;

use super::*;

pub struct AirdropSplArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub network: Network,
    pub recipient_list: Option<String>,
    pub cache_file: Option<String>,
    pub mint: Pubkey,
    pub mint_tokens: bool,
    pub boost: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SplRecipient {
    address: String,
    ata: String,
    amount: u64,
}

type Recipient = String;
type Ata = String;

pub async fn airdrop_spl(args: AirdropSplArgs) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mut jib = Jib::new(vec![keypair], args.network)?;
    let mut instructions = vec![];

    let mut recipients_lookup: HashMap<Ata, Recipient> = HashMap::new();

    let source_ata = get_associated_token_address(&jib.payer().pubkey(), &args.mint);

    let mint_account =
        spl_token::state::Mint::unpack(jib.rpc_client().get_account(&args.mint)?.data.as_slice())?;
    let decimals = mint_account.decimals;

    if args.recipient_list.is_some() && args.cache_file.is_some() {
        eprintln!("Cannot provide both a recipient list and a cache file.");
        std::process::exit(1);
    }

    // Get the current time as yyyy-mm-dd-hh-mm-ss
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d-%H-%M-%S").to_string();

    let mut cache_file_name = format!("mb-cache-airdrop-{timestamp}.json");
    let successful_tx_file_name = format!("mb-successful-airdrops-{timestamp}.json");

    let airdrop_list: HashMap<String, u64> = if let Some(list_file) = args.recipient_list {
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

    if args.mint_tokens {
        let total_tokens = airdrop_list.values().sum::<u64>();
        let total_tokens_native_units = total_tokens * 10u64.pow(decimals as u32);

        let mint_tokens_ix = spl_token::instruction::mint_to(
            &spl_token::ID,
            &args.mint,
            &source_ata,
            &jib.payer().pubkey(),
            &[],
            total_tokens_native_units,
        )?;
        send_and_confirm_tx(&args.client, &[jib.payer()], &[mint_tokens_ix])?;
    }

    if args.boost {
        jib.set_priority_fee(PRIORITY_FEE);
    }

    for (address, amount) in &airdrop_list {
        let amount_native_units = amount * 10u64.pow(decimals as u32);

        let pubkey = match Pubkey::from_str(address) {
            Ok(pubkey) => pubkey,
            Err(_) => {
                eprintln!("Invalid address: {}, skipping...", address);
                continue;
            }
        };

        let destination_ata = get_associated_token_address(&pubkey, &args.mint);

        recipients_lookup.insert(destination_ata.to_string(), pubkey.to_string());

        instructions.push(create_token_if_missing_instruction(
            &jib.payer().pubkey(),
            &destination_ata,
            &args.mint,
            &pubkey,
            &destination_ata,
        ));

        instructions.push(transfer_checked(
            &spl_token::ID,
            &source_ata,
            &args.mint,
            &destination_ata,
            &jib.payer().pubkey(),
            &[],
            amount_native_units,
            decimals,
        )?);
    }

    jib.set_instructions(instructions);
    let results = jib.hoist()?;

    if results.iter().any(|r| r.is_failure()) {
        println!("Some transactions failed. Check the {cache_file_name} cache file for details.");
    }

    let mut successes = vec![];
    let mut failures = vec![];

    results.iter().for_each(|r| {
        if r.is_failure() {
            let tx = r.transaction().unwrap(); // Transactions exist on failures.
            let account_keys = tx.message().account_keys.clone();
            let transaction_accounts = account_keys.iter().map(|k| k.to_string()).collect();

            // We iterate over all account keys and check if they are in the recipients lookup to find the
            // pubkey associated with any ATAs. Then we use the pubkey to find the address and amount pair
            // to build the airdrop list from the failures so they can be retried.
            let recipients: HashMap<String, u64> = account_keys
                .iter()
                .map(|p| p.to_string())
                .filter_map(|s| {
                    recipients_lookup.get(&s).and_then(|a| {
                        airdrop_list
                            .get_key_value(a)
                            .map(|(address, amount)| (address.clone(), *amount))
                    })
                })
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

const MPL_TOOLBOX_ID: Pubkey = pubkey!("TokExjvjJmhKaRBShsBAsbSvEWMA1AgUNK7ps4SAc2p");

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
#[rustfmt::skip]
pub enum TokenExtrasInstruction {
    /// Creates a new associated token account for the given mint and owner, if and only if
    /// the given token account does not exists and the token account is the same as the
    /// associated token account. That way, clients can ensure that, after this instruction,
    /// the token account will exists.
    ///
    /// Notice this instruction asks for both the token account and the associated token account (ATA)
    /// These may or may not be the same account. Here are all the possible cases:
    ///
    /// - Token exists and Token is ATA: Instruction succeeds.
    /// - Token exists and Token is not ATA: Instruction succeeds.
    /// - Token does not exist and Token is ATA: Instruction creates the ATA account and succeeds.
    /// - Token does not exist and Token is not ATA: Instruction fails as we cannot create a
    ///    non-ATA account without it being a signer.
    ///
    /// Note that additional checks are made to ensure that the token account provided
    /// matches the mint account and owner account provided.
    CreateTokenIfMissing,
}

fn create_token_if_missing_instruction(
    payer: &Pubkey,
    token: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    ata: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: MPL_TOOLBOX_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(*token, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new(*ata, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
        ],
        data: TokenExtrasInstruction::CreateTokenIfMissing
            .try_to_vec()
            .unwrap(),
    }
}
