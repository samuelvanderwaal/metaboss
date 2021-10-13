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
        json_file: String,

        #[structopt(short, long, default_value = ".")]
        /// Path to directory to save output files.
        output: String,
    },
    /// Get list of mint accounts for a given update authority.
    #[structopt(name = "get_mints")]
    GetMints {
        #[structopt(short, long)]
        update_authority: Option<String>,

        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        #[structopt(short, long, default_value = ".")]
        /// Path to directory to save output files.
        output: String,
    },
    /// Change an NFT's URI to point to a new metadata JSON file.
    #[structopt(name = "set_uri")]
    SetUri {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: String,

        #[structopt(short, long)]
        new_uri: String,
    },
    /// Change an NFT's URI to point to a new metadata JSON file.
    #[structopt(name = "set_uri_all")]
    SetUriAll {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Path to file containing list of mint accounts and their new URIs.
        #[structopt(short, long)]
        json_file: String,
    },
    #[structopt(name = "set_update_authority")]
    SetUpdateAuthority {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        #[structopt(short, long)]
        mint_account: String,

        #[structopt(short, long)]
        new_update_authority: String,
    },
    #[structopt(name = "set_update_authority_all")]
    SetUpdateAuthorityAll {
        #[structopt(short, long)]
        keypair: String,

        #[structopt(short, long)]
        json_file: String,
    },
    // Get candy machine holder snapshot
    #[structopt(name = "snapshot")]
    Snapshot {
        #[structopt(short, long)]
        update_authority: Option<String>,

        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        #[structopt(short, long, default_value = ".")]
        /// Path to directory to save output files.
        output: String,
    },
}
