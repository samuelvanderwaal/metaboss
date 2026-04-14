mod common;

use anyhow::Result;
use common::TestContext;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

#[test]
#[ignore]
fn test_snapshot_fvca_honors_output_flag() -> Result<()> {
    let stub_pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    // Setup test context and output directory
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("snapshot_fvca_test");
    let temp_dir_str = temp_dir.to_string_lossy().to_string();

    // Start a mock DAS RPC server that returns a valid JSON-RPC response
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let mock_url = format!("http://127.0.0.1:{}", port);

    let body = r#"{"jsonrpc":"2.0","id":1,"result":{"total":1,"limit":1000,"page":1,"items":[{"interface":"V1_NFT","id":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","content":{},"authorities":[],"compression":{},"grouping":{},"royalty":{},"creators":[{"address":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","verified":true,"share":100}],"ownership":{"delegate":null,"delegated":false,"frozen":false,"owner":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","ownership_model":"single"},"supply":{},"mutable":true,"burnt":false}]}}"#;

    // The mock server handles two requests: one with data and an empty one to end the loop
    thread::spawn(move || {
        let mut count = 0;
        while count < 2 {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0; 1024];
                let _ = stream.read(&mut buffer);

                let current_body = if count == 0 {
                    body.to_string()
                } else {
                    r#"{"jsonrpc":"2.0","id":1,"result":{"total":0,"limit":1000,"page":2,"items":[]}}"#
                        .to_string()
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    current_body.len(),
                    current_body
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
                count += 1;
            }
        }
    });

    // Run metaboss command against the mock server
    let output = ctx.run_metaboss(&[
        "snapshot",
        "fvca",
        stub_pubkey,
        "--rpc",
        &mock_url,
        "--output",
        &temp_dir_str,
    ]);
    common::assert_success(&output);

    let expected_filename = format!("{}_fvca_mints.json", stub_pubkey);

    let in_output_dir = temp_dir.join(&expected_filename).exists();
    let in_cwd = std::env::current_dir()
        .unwrap()
        .join(&expected_filename)
        .exists();

    // The file should be in the output dir
    assert!(in_output_dir, "file should be in output dir");
    assert!(!in_cwd, "file should NOT be in CWD anymore");

    // Cleanup leaked file
    if in_cwd {
        let _ = std::fs::remove_file(std::env::current_dir().unwrap().join(&expected_filename));
    }

    Ok(())
}
