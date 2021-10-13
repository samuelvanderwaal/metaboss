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
    /// Decode NFT mint account metadata into a JSON file.
    #[structopt(name = "decode")]
    Decode {
        /// List of mint accounts to decode.
        #[structopt(short, long)]
        json_file: String,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
    /// Get list of mint accounts for a given candy machine id or update authority.
    #[structopt(name = "get_mints")]
    GetMints {
        /// Update authority to filter accounts by.
        #[structopt(short, long)]
        update_authority: Option<String>,

        /// Candy Machine ID to filter accounts by.
        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
    /// Change an NFT's URI to point to a new metadata JSON file.
    #[structopt(name = "set_uri")]
    UpdateNFT {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: String,

        /// New URI with values to update the data struct with.
        #[structopt(short, long)]
        new_uri: String,
    },
    /// Change an NFT's URI to point to a new metadata JSON file.
    #[structopt(name = "set_uri_all")]
    UpdateNFTAll {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Path to file containing list of mint accounts and their new URIs.
        #[structopt(short, long)]
        json_file: String,
    },
    /// Set the update authority on a single NFT's metadata account.
    #[structopt(name = "set_update_authority")]
    SetUpdateAuthority {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: String,

        /// The new update authority to set.
        #[structopt(short, long)]
        new_update_authority: String,
    },
    /// Set the update authority on a list NFT's metadata accounts.
    #[structopt(name = "set_update_authority_all")]
    SetUpdateAuthorityAll {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Path to file containing list of mint accounts and their new update authorities.
        #[structopt(short, long)]
        json_file: String,
    },
    /// Get a snapshot of current holders of NFTs by either candy machine ID or update authority.
    #[structopt(name = "snapshot")]
    Snapshot {
        /// Update authority to filter accounts by.
        #[structopt(short, long)]
        update_authority: Option<String>,

        /// Candy Machine ID to filter accounts by.
        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
}
