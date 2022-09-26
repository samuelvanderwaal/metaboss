use super::common::*;

pub const PARALLEL_LIMIT: usize = 50;
pub type HolderResults = Vec<Result<Holder>>;

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize)]
pub struct Holder {
    pub owner_wallet: String,
    pub mint_account: String,
    pub metadata_account: String,
    pub associated_token_address: String,
}

#[derive(Debug, Serialize)]
pub struct CandyMachineProgramAccounts {
    pub config_accounts: Vec<ConfigAccount>,
    pub candy_machine_accounts: Vec<CandyMachineAccount>,
}

#[derive(Debug, Serialize)]
pub struct ConfigAccount {
    pub address: String,
    pub data_len: usize,
}

#[derive(Debug, Serialize)]
pub struct CandyMachineAccount {
    pub address: String,
    pub data_len: usize,
}

pub struct SnapshotMintsArgs {
    pub creator: Option<String>,
    pub position: usize,
    pub update_authority: Option<String>,
    pub v2: bool,
    pub allow_unverified: bool,
    pub output: String,
}

pub struct SnapshotHoldersArgs {
    pub creator: Option<String>,
    pub position: usize,
    pub update_authority: Option<String>,
    pub mint_accounts_file: Option<String>,
    pub v2: bool,
    pub allow_unverified: bool,
    pub output: String,
}

pub struct CrawlSnapshotMintsArgs {
    pub client: RpcClient,
    pub candy_machine_id: String,
    pub method: CrawlMethod,
    pub output: String,
}

#[derive(Debug)]
pub enum CrawlMethod {
    V1,
    V2,
    Authority,
}

impl FromStr for CrawlMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1" => Ok(CrawlMethod::V1),
            "v2" => Ok(CrawlMethod::V2),
            "authority" => Ok(CrawlMethod::Authority),
            _ => Err(format!("Invalid method: {}", s)),
        }
    }
}
