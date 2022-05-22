use anyhow::Result;
use retry::{delay::Exponential, retry};
use solana_client::{nonblocking::rpc_client::RpcClient as AsyncRpcClient, rpc_client::RpcClient};
use solana_sdk::{
    instruction::Instruction, signature::Keypair, signer::Signer, transaction::Transaction,
};
use std::sync::Arc;

pub fn send_and_confirm_transaction(
    client: &RpcClient,
    keypair: Keypair,
    instructions: &[Instruction],
) -> Result<String> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&keypair.pubkey()),
        &[&keypair],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );

    let sig = res?;

    println!("TxId: {}", sig);
    Ok(sig.to_string())
}

pub async fn async_send_and_confirm_transaction(
    async_client: Arc<AsyncRpcClient>,
    keypair: Arc<Keypair>,
    instructions: &[Instruction],
) -> Result<String> {
    let recent_blockhash = async_client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&keypair.pubkey()),
        &[&*keypair],
        recent_blockhash,
    );

    let sig = async_client.send_and_confirm_transaction(&tx).await?;

    Ok(sig.to_string())
}
