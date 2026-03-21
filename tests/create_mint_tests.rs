mod common;

use anyhow::Result;
use regex::Regex;
use solana_sdk::signer::Signer;

use common::{
    assert_success, create_temp_dir, decode_onchain_metadata, parse_mint_from_output,
    strip_debug_quotes, trim_null, TestContext,
};

/// Parse edition mint pubkeys from `mint editions` output which prints
/// `Edition with mint: "<pubkey>"` (Debug-formatted).
fn parse_edition_mints(output: &str) -> Vec<String> {
    let re = Regex::new(r"Edition with mint: (\S+)").expect("invalid regex");
    re.captures_iter(output)
        .filter_map(|c| c.get(1))
        .map(|m| strip_debug_quotes(m.as_str()))
        .collect()
}

// ---------------------------------------------------------------------------
// Test 1: Create a master edition NFT and mint print editions
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_create_master_edition_and_mint_editions() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("mint-editions");

    // Mint an NFT with max-editions 10 (creates master edition automatically).
    let nft_json = temp_dir.join("test_nft.json");
    ctx.create_test_nft_json(&nft_json)?;
    let nft_json_str = nft_json.to_string_lossy().to_string();

    let output = ctx.run_metaboss(&[
        "mint",
        "one",
        "-d",
        &nft_json_str,
        "-k",
        &ctx.keypair_path,
        "--max-editions",
        "10",
    ]);
    assert_success(&output);

    let raw_mint = parse_mint_from_output(&output.stdout);
    let master_mint = strip_debug_quotes(&raw_mint);

    // Mint 3 editions.
    let output = ctx.run_metaboss(&[
        "mint",
        "editions",
        "-k",
        &ctx.keypair_path,
        "-a",
        &master_mint,
        "--next-editions",
        "3",
    ]);
    assert_success(&output);

    // Parse edition mints from output.
    let edition_mints = parse_edition_mints(&output.stdout);
    assert_eq!(
        edition_mints.len(),
        3,
        "should have minted exactly 3 editions"
    );

    // Verify each edition mint exists on-chain with metadata.
    for edition_mint in &edition_mints {
        let metadata = decode_onchain_metadata(&ctx, edition_mint)?;
        assert_eq!(
            trim_null(&metadata.name),
            "Test NFT",
            "edition metadata name should match master"
        );
        assert_eq!(
            trim_null(&metadata.symbol),
            "TNFT",
            "edition metadata symbol should match master"
        );
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Mint an NFT with --immutable flag
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_one_with_immutable() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("mint-immutable");

    let nft_json = temp_dir.join("test_nft.json");
    ctx.create_test_nft_json(&nft_json)?;
    let nft_json_str = nft_json.to_string_lossy().to_string();

    let output = ctx.run_metaboss(&[
        "mint",
        "one",
        "-d",
        &nft_json_str,
        "-k",
        &ctx.keypair_path,
        "--immutable",
    ]);
    assert_success(&output);

    let raw_mint = parse_mint_from_output(&output.stdout);
    let mint = strip_debug_quotes(&raw_mint);

    // Decode metadata and verify is_mutable is false.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert!(
        !metadata.is_mutable,
        "metadata should be immutable when minted with --immutable flag"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Mint an NFT with --sign flag
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_one_with_sign() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("mint-sign");

    let nft_json = temp_dir.join("test_nft.json");
    ctx.create_test_nft_json(&nft_json)?;
    let nft_json_str = nft_json.to_string_lossy().to_string();

    let output = ctx.run_metaboss(&[
        "mint",
        "one",
        "-d",
        &nft_json_str,
        "-k",
        &ctx.keypair_path,
        "--sign",
    ]);
    assert_success(&output);

    let raw_mint = parse_mint_from_output(&output.stdout);
    let mint = strip_debug_quotes(&raw_mint);

    // After minting with --sign, the creator should already be verified.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    let creators = metadata.creators.expect("creators should be present");
    let creator = creators
        .iter()
        .find(|c| c.address == ctx.keypair.pubkey())
        .expect("test keypair should be listed as a creator");
    assert!(
        creator.verified,
        "creator should be verified when minted with --sign flag"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}
