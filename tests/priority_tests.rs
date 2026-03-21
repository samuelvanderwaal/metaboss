mod common;

use std::str::FromStr;

use anyhow::Result;
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use common::{assert_success, decode_onchain_metadata, mint_test_nft, TestContext};

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
// Test 1: Transfer an NFT with --priority high
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_transfer_asset_with_priority() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("priority-transfer");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Generate a receiver keypair (no need to fund it; the sender pays).
    let receiver = Keypair::new();
    let receiver_pubkey_str = receiver.pubkey().to_string();

    // Transfer the NFT with --priority high.
    let output = ctx.run_metaboss(&[
        "transfer",
        "asset",
        "-k",
        &ctx.keypair_path,
        "-R",
        &receiver_pubkey_str,
        "--mint",
        &mint,
        "--priority",
        "high",
    ]);
    assert_success(&output);

    // Verify the receiver now holds the token.
    let mint_pubkey = Pubkey::from_str(&mint)?;
    let receiver_ata = get_associated_token_address(&receiver.pubkey(), &mint_pubkey);
    let account = ctx
        .client
        .get_account(&receiver_ata)
        .expect("receiver ATA should exist after transfer");
    let token_account =
        TokenAccount::unpack(&account.data).expect("should unpack receiver token account");
    assert_eq!(
        token_account.amount, 1,
        "receiver should hold exactly 1 token after transfer with priority"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Sign one with --priority medium
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_sign_one_with_priority() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("priority-sign");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Before signing, the creator should be unverified.
    assert_creator_verified(&ctx, &mint, false, "should be unverified before sign");

    // Sign the metadata with --priority medium.
    let output = ctx.run_metaboss(&[
        "sign",
        "one",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--priority",
        "medium",
    ]);
    assert_success(&output);

    // After signing, the creator should be verified.
    assert_creator_verified(
        &ctx,
        &mint,
        true,
        "should be verified after sign with priority",
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Verify creator with --priority low
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_verify_creator_with_priority() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("priority-verify");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Before verifying, the creator should be unverified.
    assert_creator_verified(&ctx, &mint, false, "should be unverified before verify");

    // Verify the creator with --priority low.
    let output = ctx.run_metaboss(&[
        "verify",
        "creator",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--priority",
        "low",
    ]);
    assert_success(&output);

    // After verifying, the creator should be verified.
    assert_creator_verified(
        &ctx,
        &mint,
        true,
        "should be verified after verify creator with priority",
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Unverify creator with --priority low
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_unverify_creator_with_priority() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("priority-unverify");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // First verify the creator using sign one (without priority).
    let output = ctx.run_metaboss(&["sign", "one", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);
    assert_creator_verified(&ctx, &mint, true, "should be verified after sign one");

    // Unverify the creator with --priority low.
    let output = ctx.run_metaboss(&[
        "unverify",
        "creator",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--priority",
        "low",
    ]);
    assert_success(&output);

    // After unverifying, the creator should be unverified.
    assert_creator_verified(
        &ctx,
        &mint,
        false,
        "should be unverified after unverify creator with priority",
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 5: Default priority behavior (no --priority flag) is unchanged
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_default_priority_behavior_unchanged() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("priority-default");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Before signing, the creator should be unverified.
    assert_creator_verified(&ctx, &mint, false, "should be unverified before sign");

    // Sign without --priority flag (default "none" should work on local validator).
    let output = ctx.run_metaboss(&["sign", "one", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // After signing, the creator should be verified.
    assert_creator_verified(
        &ctx,
        &mint,
        true,
        "should be verified after sign without priority flag",
    );

    Ok(())
}
