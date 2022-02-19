use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Metaboss", about = "Metaplex NFT 'Swiss Army Knife' tool.")]
pub struct Opt {
    /// RPC endpoint url to override using the Solana config or the hard-coded default
    #[structopt(short, long, global = true)]
    pub rpc: Option<String>,

    /// Timeout to override default value of 60 seconds
    #[structopt(short, long, global = true, default_value = "90")]
    pub timeout: u64,

    /// Log level
    #[structopt(short, long, global = true, default_value = "warn")]
    pub log_level: String,

    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Burn token
    #[structopt(name = "burn")]
    Burn {
        #[structopt(subcommand)]
        burn_subcommands: BurnSubcommands,
    },
    /// Decode on-chain data into JSON format
    #[structopt(name = "decode")]
    Decode {
        #[structopt(subcommand)]
        decode_subcommands: DecodeSubcommands,
    },
    /// Derive PDAs for various account types
    Derive {
        #[structopt(subcommand)]
        derive_subcommands: DeriveSubcommands,
    },
    /// Mint new NFTs from JSON files
    #[structopt(name = "mint")]
    Mint {
        #[structopt(subcommand)]
        mint_subcommands: MintSubcommands,
    },
    /// Update various aspects of NFTs
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
    /// Sign metadata for an unverified creator
    #[structopt(name = "sign")]
    Sign {
        #[structopt(subcommand)]
        sign_subcommands: SignSubcommands,
    },
    /// Get snapshots of various blockchain states
    #[structopt(name = "snapshot")]
    Snapshot {
        #[structopt(subcommand)]
        snapshot_subcommands: SnapshotSubcommands,
    },
    /// Withdraw
    Withdraw {
        #[structopt(subcommand)]
        withdraw_subcommands: WithdrawSubcommands,
    },
}

#[derive(Debug, StructOpt)]
pub enum BurnSubcommands {
    #[structopt(name = "one")]
    One {
        /// Path to the authority & funder keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Token mint account
        #[structopt(short, long)]
        account: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum DecodeSubcommands {
    /// Decode a mint account's metadata
    #[structopt(name = "mint")]
    Mint {
        /// Single mint account to decode
        #[structopt(short, long)]
        account: Option<String>,

        /// Use this to write the full metadata struct to the output
        #[structopt(long)]
        full: bool,

        /// Path to JSON file containing a list of mint accounts to decode
        #[structopt(short, long)]
        list_file: Option<String>,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum DeriveSubcommands {
    /// Derive generic PDA from seeds and program id
    #[structopt(name = "pda")]
    Pda {
        /// Seeds to derive PDA from
        seeds: String,
        /// Program id to derive PDA from
        program_id: String,
    },
    /// Derive Metadata PDA
    #[structopt(name = "metadata")]
    Metadata { mint_account: String },
    /// Derive Edition PDA
    #[structopt(name = "edition")]
    Edition { mint_account: String },
    /// Derive CMV2 PDA
    #[structopt(name = "cmv2-creator")]
    CMV2Creator { candy_machine_id: String },
}

#[derive(Debug, StructOpt)]
pub enum MintSubcommands {
    /// Mint a single NFT from a JSON file
    #[structopt(name = "one")]
    One {
        /// Path to the update_authority keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Receiving address, if different from update authority.
        #[structopt(short, long)]
        receiver: Option<String>,

        /// On-chain formatted metadata for the new NFT
        #[structopt(short = "d", long)]
        nft_data_file: Option<String>,

        /// Link to external metadata to use to create the NFT
        #[structopt(short = "u", long)]
        external_metadata_uri: Option<String>,

        /// Mint the NFT with immutable data fields
        #[structopt(short, long)]
        immutable: bool,

        /// Mint the NFT with primary_sale_happened set to true
        #[structopt(short, long)]
        primary_sale_happened: bool,

        /// Sign NFT after minting it
        #[structopt(long)]
        sign: bool,
    },
    #[structopt(name = "list")]
    /// Mint a list of NFTs from a directory of JSON files
    List {
        /// Path to the update_authority keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Receiving address, if different from update authority
        #[structopt(short, long)]
        receiver: Option<String>,

        /// Directory of on-chain formatted metadata files for the new NFTs
        #[structopt(short = "d", long)]
        nft_data_dir: Option<String>,

        /// List of external metadata links to use to create the NFTs
        #[structopt(short = "u", long)]
        external_metadata_uris: Option<String>,

        /// Mint the NFTs with immutable data fields
        #[structopt(short, long)]
        immutable: bool,

        /// Mint the NFTs with primary_sale_happened set to true
        #[structopt(short, long)]
        primary_sale_happened: bool,

        /// Sign NFTs after minting them
        #[structopt(long)]
        sign: bool,
    },
}

#[derive(Debug, StructOpt)]
pub enum SetSubcommands {
    /// Set primary sale happened to true
    #[structopt(name = "primary-sale-happened")]
    PrimarySaleHappened {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,
    },
    /// Set update authority to a new account
    #[structopt(name = "update-authority")]
    UpdateAuthority {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,

        /// New update authority address
        #[structopt(short = "u", long)]
        new_update_authority: String,
    },
    /// Set update authority on multiple accounts to a new account
    #[structopt(name = "update-authority-all")]
    UpdateAuthorityAll {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Path to JSON mint accounts file
        #[structopt(short = "a", long)]
        mint_accounts_file: String,

        /// New update authority address
        #[structopt(short = "u", long)]
        new_update_authority: String,
    },
    /// Set is-mutable to false, preventing any future updates to the NFT
    #[structopt(name = "immutable")]
    Immutable {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,
    },
    ImmutableAll {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Path to JSON mint accounts file
        #[structopt(short, long)]
        mint_accounts_file: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum SignSubcommands {
    /// Sign the metadata for a single mint account
    #[structopt(name = "one")]
    One {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Mint account to sign
        #[structopt(short, long)]
        account: String,
    },
    /// Sign all metadata from a JSON list or for a given candy machine id
    #[structopt(name = "all")]
    All {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Candy Machine ID to filter accounts by
        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        /// Candy machine v2 id
        #[structopt(long = "v2")]
        v2: bool,

        /// Path to JSON file with list of mint accounts to sign
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

        /// Candy machine v2 id
        #[structopt(long = "v2")]
        v2: bool,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
    ///Snapshot all candy machine config and state accounts for a given update_authority
    #[structopt(name = "cm-accounts")]
    CMAccounts {
        /// Update authority to filter accounts by.
        #[structopt(short, long)]
        update_authority: String,

        /// Path to directory to save output files.
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
    /// Snapshot all mint accounts for a given candy machine id or update authority
    #[structopt(name = "mints")]
    Mints {
        /// Candy Machine ID to filter accounts by
        #[structopt(short, long)]
        candy_machine_id: Option<String>,

        /// Update authority to filter accounts by.
        #[structopt(short, long)]
        update_authority: Option<String>,

        /// Candy machine v2 id
        #[structopt(long = "v2")]
        v2: bool,

        /// Path to directory to save output file
        #[structopt(short, long, default_value = ".")]
        output: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum UpdateSubcommands {
    /// Update the data struct on a NFT
    #[structopt(name = "data")]
    Data {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,

        /// Path to JSON file containing new data
        #[structopt(short, long)]
        new_data_file: String,
    },
    /// Update the data struct on a list of NFTs
    #[structopt(name = "data-all")]
    DataAll {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Path to directory containing JSON files with new data
        #[structopt(short, long)]
        data_dir: String,
    },
    /// Update the metadata URI, keeping the rest of the data the same
    #[structopt(name = "uri")]
    Uri {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// Mint account of corresponding metadata to update
        #[structopt(short, long)]
        account: String,

        /// New uri
        #[structopt(short = "u", long)]
        new_uri: String,
    },
    /// Update the metadata URI on a list of mint accounts
    #[structopt(name = "uri-all")]
    UriAll {
        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,

        /// JSON file with list of mint accounts and new URIs
        #[structopt(short = "u", long)]
        json_file: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum WithdrawSubcommands {
    /// Withdraw funds from a candy machine v2
    #[structopt(name = "cm-v2")]
    CMV2 {
        /// Candy Machine V2 ID
        candy_machine_id: String,

        /// Path to the creator's keypair file
        #[structopt(short, long)]
        keypair: String,
    },
}
