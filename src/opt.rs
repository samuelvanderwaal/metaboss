use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Metaboss",
    about = "Metaplex NFT-standard Swiss army knife tool."
)]
pub struct Opt {
    #[structopt(short, long, default_value = "https://api.devnet.solana.com")]
    pub rpc: String,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "decode")]
    Decode {
        /// List of mint accounts to decode.
        #[structopt(short, long)]
        mint_accounts: Vec<String>,

        #[structopt(short, long, default_value = ".")]
        /// Path to directory to save output files.
        output: String,
    },
}
