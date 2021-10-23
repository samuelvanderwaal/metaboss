use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Metaboss", about = "Metaplex NFT 'Swiss Army Knife' tool.")]
pub struct Opt {
    #[structopt(short, long)]
    pub rpc: Option<String>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Decode a single NFT mint account metadata into a JSON file.
    #[structopt(name = "decode")]
    Decode {
        /// Mint account to decode.
        #[structopt(short, long)]
        mint_account: String,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
    /// Decode a list of NFT mint accounts into JSON files.
    #[structopt(name = "decode_all")]
    DecodeAll {
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
    /// Get all candy machine state and config accounts for a given update authority.
    #[structopt(name = "get_cm_accounts")]
    GetCMAccounts {
        /// Update authority to filter accounts by.
        #[structopt(short, long)]
        update_authority: String,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
    /// Mint a new NFT.
    #[structopt(name = "mint_nft")]
    MintNFT {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// On-chain formatted metadata for the new NFT.
        #[structopt(short, long)]
        json_file: String,
    },
    /// Update all data fields on a NFT.
    #[structopt(name = "update_nft")]
    UpdateNFT {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: String,

        /// File containing new NFT data
        #[structopt(short, long)]
        json_file: String,
    },
    /// Change an NFT's URI to point to a new metadata JSON file
    #[structopt(name = "set_new_uri")]
    SetNewURI {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: String,

        /// New URI
        #[structopt(short, long)]
        new_uri: String,
    },
    /// Set primary_sale_happened on the NFT.
    #[structopt(name = "set_primary_sale_happened")]
    SetPrimarySaleHappened {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: String,
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
    /// Sign all metadata accounts associated with a candy machine id with provided creator key.
    #[structopt(name = "sign")]
    Sign {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Candy Machine ID to filter accounts by.
        #[structopt(short, long, required_unless = "mint-account")]
        candy_machine_id: Option<String>,

        /// Mint account to update.
        #[structopt(short, long)]
        mint_account: Option<String>,
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
