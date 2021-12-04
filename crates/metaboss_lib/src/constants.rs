use lazy_static::lazy_static;
use std::sync::RwLock;

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_CREATOR_LEN: usize = 32 + 1 + 1;

pub const METAPLEX_PROGRAM_ID: &'static str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const CANDY_MACHINE_PROGRAM_ID: &'static str = "cndyAnrLdpjq1Ssp1z8xxDsB8dxe7u4HL5Nxi2K5WXZ";

pub const DEFAULT_RPC_DELAY_MS: u64 = 300;

pub const PUBLIC_RPC_URLS: &'static [&'static str] = &[
    "https://api.devnet.solana.com",
    "https://api.testnet.solana.com",
    "https://api.mainnet-beta.solana.com",
    "https://solana-api.projectserum.com",
];

pub const MAX_REQUESTS: u64 = 40;
pub const TIME_PER_MAX_REQUESTS_NS: u64 = 10_000_000_000;
pub const TIME_BUFFER_NS: u32 = 50_000_000;

// Delay in milliseconds between RPC requests
pub const RATE_LIMIT: u64 = 500;

lazy_static! {
    pub static ref USE_RATE_LIMIT: RwLock<bool> = RwLock::new(false);
}
