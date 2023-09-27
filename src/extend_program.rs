use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{bpf_loader_upgradeable::extend_program, signer::Signer};

use crate::{
    update::{parse_keypair, parse_solana_config},
    utils::send_and_confirm_transaction,
};

pub fn process_extend_program(
    client: RpcClient,
    keypair_path: Option<String>,
    program_address: Pubkey,
    additional_bytes: u32,
) -> Result<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let ix = extend_program(&program_address, Some(&keypair.pubkey()), additional_bytes);
    send_and_confirm_transaction(&client, keypair, &[ix])?;

    Ok(())
}
