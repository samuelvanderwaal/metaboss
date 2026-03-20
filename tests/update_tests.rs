mod common;

use std::io::Write;
use std::time::Instant;

use anyhow::Result;
use metaboss_lib::decode::decode_metadata_from_mint;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use common::{assert_success, parse_mint_from_output, TestContext};

/// Strip surrounding quotes from a string that was printed with Rust Debug
/// formatting (e.g. `"J7abc..."` -> `J7abc...`).
fn strip_debug_quotes(s: &str) -> String {
    s.trim_matches('"').to_string()
}

/// Create a unique temporary directory for test artifacts.
fn create_temp_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "metaboss-update-{}-{}-{}",
        label,
        std::process::id(),
        Instant::now().elapsed().as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

/// Helper: mint a test NFT using `mint one`, returning the stripped mint address.
fn mint_test_nft(ctx: &TestContext, temp_dir: &std::path::Path) -> Result<String> {
    let nft_json = temp_dir.join("test_nft.json");
    ctx.create_test_nft_json(&nft_json)?;

    let nft_json_str = nft_json.to_string_lossy().to_string();
    let output = ctx.run_metaboss(&["mint", "one", "-d", &nft_json_str, "-k", &ctx.keypair_path]);
    assert_success(&output);

    let raw_mint = parse_mint_from_output(&output.stdout);
    Ok(strip_debug_quotes(&raw_mint))
}

/// Helper: decode on-chain metadata for a given mint address.
fn decode_onchain_metadata(
    ctx: &TestContext,
    mint_str: &str,
) -> Result<mpl_token_metadata::accounts::Metadata> {
    let metadata = decode_metadata_from_mint(&ctx.client, mint_str.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to decode metadata: {:?}", e))?;
    Ok(metadata)
}

/// Trim null bytes from a metadata string field.
fn trim_null(s: &str) -> &str {
    s.trim_matches(char::from(0))
}

// ---------------------------------------------------------------------------
// Test 1: Mint an NFT and update its symbol
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_symbol() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("symbol");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Verify initial symbol.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(trim_null(&metadata.symbol), "TNFT");

    // Update the symbol.
    let output = ctx.run_metaboss(&[
        "update",
        "symbol",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-symbol",
        "NEWT",
    ]);
    assert_success(&output);

    // Verify symbol was updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        trim_null(&metadata.symbol),
        "NEWT",
        "symbol should be updated to NEWT"
    );

    // Other fields should remain unchanged.
    assert_eq!(trim_null(&metadata.name), "Test NFT");
    assert_eq!(metadata.seller_fee_basis_points, 100);

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Mint an NFT and update its seller fee basis points
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_seller_fee_basis_points() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("sfbp");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Verify initial seller fee basis points.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(metadata.seller_fee_basis_points, 100);

    // Update the seller fee basis points.
    let output = ctx.run_metaboss(&[
        "update",
        "sfbp",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-sfbp",
        "500",
    ]);
    assert_success(&output);

    // Verify seller fee basis points was updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        metadata.seller_fee_basis_points, 500,
        "seller_fee_basis_points should be updated to 500"
    );

    // Other fields should remain unchanged.
    assert_eq!(trim_null(&metadata.name), "Test NFT");
    assert_eq!(trim_null(&metadata.symbol), "TNFT");

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Mint an NFT and update its creators
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_creators() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("creators");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Generate a second keypair to use as a new creator.
    let new_creator = Keypair::new();
    let new_creator_pubkey = new_creator.pubkey().to_string();

    // Format: address:share:verified
    let new_creators_arg = format!("{}:100:false", new_creator_pubkey);

    // Update the creators.
    let output = ctx.run_metaboss(&[
        "update",
        "creators",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-creators",
        &new_creators_arg,
    ]);
    assert_success(&output);

    // Verify creators were updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    let creators = metadata
        .creators
        .expect("creators should be present after update");

    assert_eq!(creators.len(), 1, "should have exactly one creator");
    assert_eq!(
        creators[0].address.to_string(),
        new_creator_pubkey,
        "creator address should match the new creator"
    );
    assert_eq!(creators[0].share, 100, "creator share should be 100");
    assert!(!creators[0].verified, "creator should be unverified");

    // Other fields should remain unchanged.
    assert_eq!(trim_null(&metadata.name), "Test NFT");
    assert_eq!(trim_null(&metadata.symbol), "TNFT");
    assert_eq!(metadata.seller_fee_basis_points, 100);

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Mint an NFT and update its entire data struct via a JSON file
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_data() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("data");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Generate a new creator keypair for the updated data.
    let new_creator = Keypair::new();
    let new_creator_pubkey = new_creator.pubkey().to_string();

    // Write a JSON file with the new data struct.
    let new_data = serde_json::json!({
        "name": "Updated NFT",
        "symbol": "UPDT",
        "uri": "https://example.com/updated-metadata.json",
        "seller_fee_basis_points": 250,
        "creators": [
            {
                "address": new_creator_pubkey,
                "verified": false,
                "share": 100
            }
        ]
    });

    let data_file = temp_dir.join("new_data.json");
    let mut file = std::fs::File::create(&data_file)?;
    file.write_all(serde_json::to_string_pretty(&new_data)?.as_bytes())?;

    let data_file_str = data_file.to_string_lossy().to_string();

    // Update the data.
    let output = ctx.run_metaboss(&[
        "update",
        "data",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-data-file",
        &data_file_str,
    ]);
    assert_success(&output);

    // Verify all fields were updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;

    assert_eq!(
        trim_null(&metadata.name),
        "Updated NFT",
        "name should be updated"
    );
    assert_eq!(
        trim_null(&metadata.symbol),
        "UPDT",
        "symbol should be updated"
    );
    assert_eq!(
        trim_null(&metadata.uri),
        "https://example.com/updated-metadata.json",
        "uri should be updated"
    );
    assert_eq!(
        metadata.seller_fee_basis_points, 250,
        "seller_fee_basis_points should be updated to 250"
    );

    let creators = metadata
        .creators
        .expect("creators should be present after update");
    assert_eq!(creators.len(), 1, "should have exactly one creator");
    assert_eq!(
        creators[0].address.to_string(),
        new_creator_pubkey,
        "creator address should match the new creator"
    );
    assert_eq!(creators[0].share, 100);
    assert!(!creators[0].verified);

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}
