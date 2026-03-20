mod common;

use std::time::Instant;

use anyhow::Result;
use metaboss_lib::decode::decode_metadata_from_mint;
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
        "metaboss-verify-{}-{}-{}",
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

/// Helper: decode on-chain metadata for a given mint address and return the
/// Metadata struct.
fn decode_onchain_metadata(
    ctx: &TestContext,
    mint_str: &str,
) -> Result<mpl_token_metadata::accounts::Metadata> {
    let metadata = decode_metadata_from_mint(&ctx.client, mint_str.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to decode metadata: {:?}", e))?;
    Ok(metadata)
}

/// Helper: assert the verified status of the test keypair creator on a given mint.
fn assert_creator_verified(ctx: &TestContext, mint: &str, expected_verified: bool, context: &str) {
    let metadata = decode_onchain_metadata(ctx, mint).expect("should decode metadata");
    let creators = metadata.creators.expect("creators should be present");
    let creator = creators
        .iter()
        .find(|c| c.address == ctx.keypair.pubkey())
        .expect("test keypair should be listed as a creator");
    assert_eq!(
        creator.verified, expected_verified,
        "creator verified mismatch: {}",
        context
    );
}

// ---------------------------------------------------------------------------
// Test 1: Verify a creator on an NFT
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_verify_creator() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("verify");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Before verifying, the creator should be unverified.
    assert_creator_verified(&ctx, &mint, false, "should be unverified after mint");

    // Verify the creator.
    let output = ctx.run_metaboss(&["verify", "creator", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // After verifying, the creator should be verified.
    assert_creator_verified(&ctx, &mint, true, "should be verified after verify creator");

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Unverify a previously verified creator
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_unverify_creator() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("unverify");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // First verify the creator using `sign one`.
    let output = ctx.run_metaboss(&["sign", "one", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // Confirm the creator is verified.
    assert_creator_verified(&ctx, &mint, true, "should be verified after sign one");

    // Unverify the creator.
    let output = ctx.run_metaboss(&["unverify", "creator", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // After unverifying, the creator should be unverified.
    assert_creator_verified(
        &ctx,
        &mint,
        false,
        "should be unverified after unverify creator",
    );

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Full verify/unverify roundtrip
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_verify_and_unverify_roundtrip() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("roundtrip");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Step 1: Creator should be unverified after mint.
    assert_creator_verified(&ctx, &mint, false, "step 1: unverified after mint");

    // Step 2: Verify the creator.
    let output = ctx.run_metaboss(&["verify", "creator", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);
    assert_creator_verified(&ctx, &mint, true, "step 2: verified after verify creator");

    // Step 3: Unverify the creator.
    let output = ctx.run_metaboss(&["unverify", "creator", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);
    assert_creator_verified(
        &ctx,
        &mint,
        false,
        "step 3: unverified after unverify creator",
    );

    // Step 4: Verify the creator again.
    let output = ctx.run_metaboss(&["verify", "creator", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);
    assert_creator_verified(
        &ctx,
        &mint,
        true,
        "step 4: verified again after second verify creator",
    );

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}
