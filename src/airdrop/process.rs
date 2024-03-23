use structopt::StructOpt;

use super::*;

#[derive(Debug, StructOpt)]
pub enum AirdropSubcommands {
    /// Airdrop SOL (experimental)
    #[structopt(name = "sol")]
    Sol {
        /// Path to the owner keypair file
        #[structopt(short, long)]
        keypair: Option<String>,

        /// Path to the mint list file
        #[structopt(short = "L", long)]
        recipient_list: Option<String>,

        /// Cache file
        #[structopt(short, long)]
        cache_file: Option<String>,

        /// Rate limit in requests per second; defaults to 10
        #[structopt(short = "R", long)]
        rate_limit: Option<u64>,

        /// Priority of the transaction: higher priority costs more.
        #[structopt(short = "P", long, default_value = "none")]
        priority: Priority,
    },
    /// Airdrop SPL tokens (experimental)
    #[structopt(name = "spl")]
    Spl {
        /// Path to the owner keypair file
        #[structopt(short, long)]
        keypair: Option<String>,

        /// Path to the mint list file
        #[structopt(short = "L", long)]
        recipient_list: Option<String>,

        /// Cache file
        #[structopt(short, long)]
        cache_file: Option<String>,

        /// Mint from the SPL token mint
        #[structopt(short, long)]
        mint: Pubkey,

        #[structopt(long)]
        mint_tokens: bool,

        /// Rate limit in requests per second; defaults to 10
        #[structopt(short = "R", long)]
        rate_limit: Option<u64>,

        /// Priority of the transaction: higher priority costs more.
        #[structopt(short = "P", long, default_value = "none")]
        priority: Priority,
    },
    /// Convert the bin cache file to json for readability
    ReadCache {
        /// Path to the cache file
        cache_file: String,

        /// Print errors to std out in addition to converting the cache file to json
        #[structopt(long)]
        errors: bool,
    },
}

pub async fn process_airdrop(client: RpcClient, commands: AirdropSubcommands) -> Result<()> {
    match commands {
        AirdropSubcommands::Sol {
            keypair,
            recipient_list,
            cache_file,
            priority,
            rate_limit,
        } => {
            airdrop_sol(AirdropSolArgs {
                client,
                keypair,
                recipient_list,
                cache_file,
                priority,
                rate_limit,
            })
            .await
        }
        AirdropSubcommands::Spl {
            keypair,
            recipient_list,
            cache_file,
            mint,
            mint_tokens,
            priority,
            rate_limit,
        } => {
            airdrop_spl(AirdropSplArgs {
                client,
                keypair,
                recipient_list,
                cache_file,
                mint,
                mint_tokens,
                priority,
                rate_limit,
            })
            .await
        }
        AirdropSubcommands::ReadCache { cache_file, errors } => {
            let path = std::path::Path::new(&cache_file);
            let file = File::open(path)?;
            let cache: Vec<JibFailedTransaction> = bincode::deserialize_from(file)?;

            // Convert to JSON
            let json_filename = path.with_extension("json");
            let pb = ProgressBar::new_spinner();
            pb.set_message("Writing cache file...");
            pb.enable_steady_tick(100);

            let cache_file = std::fs::File::create(json_filename)?;
            serde_json::to_writer(cache_file, &cache)?;
            pb.finish_and_clear();

            if errors {
                for tx in cache {
                    println!("{:?}", tx.error);
                }
            }
            Ok(())
        }
    }
}
