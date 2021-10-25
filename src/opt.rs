use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Metaboss", about = "Metaplex NFT 'Swiss Army Knife' tool.")]
pub struct Opt {
    /// RPC endpoint url to override using the Solana config or the hard-coded default.
    #[structopt(short, long)]
    pub rpc: Option<String>,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Decode on-chain data into JSON format.
    #[structopt(name = "decode")]
    Decode {
        #[structopt(subcommand)]
        decode_subcommands: DecodeSubcommands,
    },
    /// Mint new NFTs from JSON files.
    #[structopt(name = "mint")]
    Mint {
        #[structopt(subcommand)]
        mint_subcommands: MintSubcommands,
    },
    /// Update various aspects of NFTs.
    #[structopt(name = "update")]
    Update {
        #[structopt(subcommand)]
        update_subcommands: UpdateSubcommands,
    },
    /// Set non-Data struct values for a NFT
    #[structopt(name = "set")]
    Set {
        #[structopt(subcommand)]
        set_subcommands: SetSubcommands,
    },
    /// Sign metadata for an unverified creator.
    #[structopt(name = "sign")]
    Sign {
        #[structopt(subcommand)]
        sign_subcommands: SignSubcommands,
    },
    /// Get snapshots of various blockchain states.
    #[structopt(name = "snapshot")]
    Snapshot {
        #[structopt(subcommand)]
        snapshot_subcommands: SnapshotSubcommands,
    },
}

#[derive(Debug, StructOpt)]
pub enum DecodeSubcommands {
    /// Decode a mint account's metadata
    #[structopt(name = "mint")]
    Mint {
        /// Single mint account to decode.
        #[structopt(short, long)]
        account: Option<String>,

        /// Path to JSON file containing a list of mint accounts to decode.
        #[structopt(short, long)]
        list_file: Option<String>,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum MintSubcommands {
    #[structopt(name = "one")]
    One {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Receiving address, if different from update authority.
        #[structopt(short, long)]
        receiver: Option<String>,

        /// On-chain formatted metadata for the new NFT.
        #[structopt(short = "d", long)]
        nft_data_file: String,
    },
    #[structopt(name = "list")]
    List {
        /// Path to the update_authority keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Receiving address, if different from update authority.
        #[structopt(short, long)]
        receiver: Option<String>,

        /// Directory of on-chain formatted metadata files for the new NFTs.
        #[structopt(short = "d", long)]
        nft_data_dir: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum UpdateSubcommands {
    /// Update the data struct on a NFT
    #[structopt(name = "data")]
    Data {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,

        /// Path to JSON file containing new data
        #[structopt(short, long)]
        new_data_file: String,
    },
    /// Update the metadata URI, keeping the rest of the data the same
    #[structopt(name = "uri")]
    Uri {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,

        /// New uri
        #[structopt(short = "u", long)]
        new_uri: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum SetSubcommands {
    /// Set primary sale happened to true
    #[structopt(name = "primary-sale-happened")]
    PrimarySaleHappened {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,
    },
    /// Set update authority to a new account
    #[structopt(name = "update-authority")]
    UpdateAuthority {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,

        /// New update authority address
        #[structopt(short = "u", long)]
        new_update_authority: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum SignSubcommands {
    #[structopt(name = "one")]
    One {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to sign.
        #[structopt(short, long)]
        account: String,
    },
    #[structopt(name = "all")]
    All {
        /// Path to the creator's keypair file.
        #[structopt(short, long)]
        keypair: String,

        /// Candy Machine ID to filter accounts by.
        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        /// Directory of mint accounts to sign.
        #[structopt(short, long)]
        mint_accounts_file: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
pub enum SnapshotSubcommands {
    /// Snapshot all current holders of NFTs by candy_machine_id or update_authority
    #[structopt(name = "holders")]
    Holders {
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
    ///Snapshot all candy machine config and state accounts by update_authority
    #[structopt(name = "cm-accounts")]
    CMAccounts {
        /// Update authority to filter accounts by.
        #[structopt(short, long)]
        update_authority: String,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
}
