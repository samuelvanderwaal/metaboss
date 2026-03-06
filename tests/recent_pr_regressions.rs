use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use metaboss::mint::mint_editions;
use metaboss_lib::data::Priority;
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

fn write_temp_keypair_file() -> String {
    let keypair = Keypair::new();
    let key_bytes = keypair.to_bytes().to_vec();
    let file_name = format!(
        "metaboss-test-keypair-{}.json",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    );
    let path = std::env::temp_dir().join(file_name);
    fs::write(&path, serde_json::to_string(&key_bytes).unwrap()).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn mint_editions_specific_editions_returns_error_on_failures() {
    let keypair_path = write_temp_keypair_file();
    let client = RpcClient::new("http://localhost:8899".to_string());

    let result = mint_editions(
        &client,
        Some(keypair_path.clone()),
        "invalid-mint".to_string(),
        &None,
        None,
        Some(vec![1, 2]),
        Priority::None,
    );

    let _ = fs::remove_file(keypair_path);

    assert!(
        result.is_err(),
        "mint_editions should report errors when specific edition minting fails"
    );
}
