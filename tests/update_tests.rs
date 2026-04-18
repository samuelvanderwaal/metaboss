mod common;

use std::io::Write;

use anyhow::Result;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use common::{assert_success, decode_onchain_metadata, mint_test_nft, trim_null, TestContext};

// ---------------------------------------------------------------------------
// Test 1: Mint an NFT and update its symbol
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_symbol() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update-symbol");
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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Mint an NFT and update its name
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_name() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update-name");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Verify initial name.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(trim_null(&metadata.name), "Test NFT");

    // Update the name.
    let output = ctx.run_metaboss(&[
        "update",
        "name",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-name",
        "Updated Name",
    ]);
    assert_success(&output);

    // Verify name was updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        trim_null(&metadata.name),
        "Updated Name",
        "name should be updated to 'Updated Name'"
    );

    // Other fields should remain unchanged.
    assert_eq!(trim_null(&metadata.symbol), "TNFT");
    assert_eq!(metadata.seller_fee_basis_points, 100);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Mint an NFT and update its seller fee basis points
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_seller_fee_basis_points() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update-sfbp");
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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Mint an NFT and update its creators
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_creators() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update-creators");
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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 5: Mint an NFT and update its entire data struct via a JSON file
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_data() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update-data");
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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 6: Mint an NFT and execute update creators-all with --append flag
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_update_creators_all_append() -> Result<()> {
    // Initialize Context
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update-creators-all-append");

    // Mint test NFT (Assigns default creator A)
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Write mint address to a temp list file
    let mint_list_file = temp_dir.join("append_test_mints.json");
    let mut file = std::fs::File::create(&mint_list_file)?;
    file.write_all(serde_json::to_string(&vec![&mint])?.as_bytes())?;
    let mint_list_file_str = mint_list_file.to_string_lossy().to_string();

    // Generate a new keypair for creator B
    let new_creator = Keypair::new();
    let new_creator_pubkey = new_creator.pubkey().to_string();
    let new_creators_arg = format!("{}:0:false", new_creator_pubkey);

    // Run update creators-all --append with creator B
    let output = ctx.run_metaboss(&[
        "update",
        "creators-all",
        "-k",
        &ctx.keypair_path,
        "-L",
        &mint_list_file_str,
        "--new-creators",
        &new_creators_arg,
        "--append",
    ]);
    assert_success(&output);

    // Decode and assert creators
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    let creators = metadata
        .creators
        .expect("creators should be present after update");

    // The --append flag is respected and creator B is appended to creator A,
    // so length is 2.
    assert_eq!(creators.len(), 2, "--append works, both creators present");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 7: Resume update uri-all with a cache file containing a mint not present
// in the new URIs file. This should currently panic.
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_uri_all_resume_missing_mint_panics() -> Result<()> {
    // Initialize context
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("uri-all-resume-panic");

    // Mint a real NFT
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Create cache file with the mint
    let cache_file = temp_dir.join("resume_cache.json");
    let cache_data = serde_json::json!({
        mint: {
            "error": "Simulated failure"
        }
    });
    std::fs::write(&cache_file, serde_json::to_string(&cache_data)?)?;
    let cache_file_str = cache_file.to_string_lossy().to_string();

    // Create new_uris_file that is empty (mint removed by user)
    let new_uris_file = temp_dir.join("new_uris.json");
    let new_uris_data: Vec<serde_json::Value> = vec![];
    std::fs::write(&new_uris_file, serde_json::to_string(&new_uris_data)?)?;
    let new_uris_file_str = new_uris_file.to_string_lossy().to_string();

    // Run metaboss update uri-all
    let output = ctx.run_metaboss(&[
        "update",
        "uri-all",
        "-k",
        &ctx.keypair_path,
        "--new-uris-file",
        &new_uris_file_str,
        "--cache-file",
        &cache_file_str,
    ]);

    // Current buggy behavior: process should have panicked
    assert!(!output.success, "bug: process should have panicked/crashed");

    assert!(
        output.stderr.contains("panicked") || output.stderr.contains("unwrap"),
        "bug: expected panic in stderr, got: {}",
        output.stderr
    );

    Ok(())
}
