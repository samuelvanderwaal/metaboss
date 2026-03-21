mod common;

use anyhow::Result;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use common::{
    assert_success, create_temp_dir, decode_onchain_metadata, mint_test_nft, TestContext,
};

// ---------------------------------------------------------------------------
// Test 1: Mint an NFT and set it as immutable
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_set_immutable() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("set-immutable");
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
    let temp_dir = create_temp_dir("set-secondary-sale");
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
    let temp_dir = create_temp_dir("set-update-authority");
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
    let temp_dir = create_temp_dir("set-immutable-no-update");
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
