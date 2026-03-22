use super::*;

pub struct AirdropSolArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub recipient_list: Option<String>,
    pub cache_file: Option<String>,
    pub priority: Priority,
    pub rate_limit: Option<u64>,
}

pub async fn airdrop_sol(_args: AirdropSolArgs) -> Result<()> {
    anyhow::bail!("Airdrop SOL is temporarily unavailable during Solana v2 migration. The jib dependency needs to be updated.")
}
