mod common;

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
        "metaboss-set-{}-{}-{}",
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

// ---------------------------------------------------------------------------
// Test 1: Mint an NFT and set it as immutable
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_set_immutable() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("immutable");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Verify is_mutable is true initially.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert!(
        metadata.is_mutable,
        "NFT should be mutable immediately after minting"
    );

    // Set the NFT as immutable.
    let output = ctx.run_metaboss(&["set", "immutable", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // Verify is_mutable is now false.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert!(
        !metadata.is_mutable,
        "NFT should be immutable after set immutable"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Mint an NFT and set primary_sale_happened
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_set_secondary_sale() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("secondary-sale");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Verify primary_sale_happened is false initially.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert!(
        !metadata.primary_sale_happened,
        "primary_sale_happened should be false after minting"
    );

    // Set secondary sale (primary_sale_happened = true).
    let output = ctx.run_metaboss(&[
        "set",
        "secondary-sale",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
    ]);
    assert_success(&output);

    // Verify primary_sale_happened is now true.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert!(
        metadata.primary_sale_happened,
        "primary_sale_happened should be true after set secondary-sale"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Mint an NFT and change its update authority
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_set_update_authority() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("update-authority");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Verify initial update authority is the test keypair.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        metadata.update_authority,
        ctx.keypair.pubkey(),
        "update_authority should be the test keypair initially"
    );

    // Generate a new keypair to become the new update authority.
    let new_authority = Keypair::new();
    let new_authority_pubkey = new_authority.pubkey().to_string();

    // Set the new update authority.
    let output = ctx.run_metaboss(&[
        "set",
        "update-authority",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-update-authority",
        &new_authority_pubkey,
    ]);
    assert_success(&output);

    // Verify update authority changed.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        metadata.update_authority,
        new_authority.pubkey(),
        "update_authority should be the new authority after set update-authority"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Set immutable prevents further updates
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_set_immutable_prevents_further_updates() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("immutable-no-update");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Set the NFT as immutable.
    let output = ctx.run_metaboss(&["set", "immutable", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // Try to update the name -- this should fail because the NFT is immutable.
    let output = ctx.run_metaboss(&[
        "update",
        "name",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-name",
        "Should Fail",
    ]);
    assert!(
        !output.success,
        "Updating an immutable NFT should fail.\nstdout:\n{}\nstderr:\n{}",
        output.stdout, output.stderr,
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}
