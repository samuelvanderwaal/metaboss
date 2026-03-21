mod common;

use std::str::FromStr;

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;

use common::{
    assert_success, create_temp_dir, mint_test_nft, parse_mint_from_output, strip_debug_quotes,
    TestContext,
};

/// Helper: mint a test NFT with --max-editions flag.
fn mint_test_nft_with_editions(
    ctx: &TestContext,
    temp_dir: &std::path::Path,
    max_editions: u64,
) -> Result<String> {
    let nft_json = temp_dir.join("test_nft.json");
    ctx.create_test_nft_json(&nft_json)?;

    let nft_json_str = nft_json.to_string_lossy().to_string();
    let max_editions_str = max_editions.to_string();
    let output = ctx.run_metaboss(&[
        "mint",
        "one",
        "-d",
        &nft_json_str,
        "-k",
        &ctx.keypair_path,
        "--max-editions",
        &max_editions_str,
    ]);
    assert_success(&output);

    let raw_mint = parse_mint_from_output(&output.stdout);
    Ok(strip_debug_quotes(&raw_mint))
}

// ---------------------------------------------------------------------------
// Test 1: Decode a raw SPL mint account
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_decode_mint_account() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("decode-mint-account");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // `decode mint-account -a <mint>` prints the SPL Mint struct to stdout.
    let output = ctx.run_metaboss(&["decode", "mint-account", "-a", &mint]);
    assert_success(&output);

    let stdout = &output.stdout;
    // The output is a Debug-formatted spl_token::state::Mint. Verify key fields.
    assert!(
        stdout.contains("supply"),
        "decode mint-account output should contain 'supply', got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("decimals"),
        "decode mint-account output should contain 'decimals', got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("mint_authority"),
        "decode mint-account output should contain 'mint_authority', got:\n{}",
        stdout
    );

    // NFTs have supply 1 and 0 decimals.
    assert!(
        stdout.contains("1") && stdout.contains("0"),
        "NFT mint should have supply=1 and decimals=0, got:\n{}",
        stdout
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Decode a token account (ATA)
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_decode_token_account() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("decode-token-account");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Derive the ATA for the minter.
    let mint_pubkey = Pubkey::from_str(&mint)?;
    let ata = get_associated_token_address(&ctx.keypair.pubkey(), &mint_pubkey);
    let ata_str = ata.to_string();

    // `decode token-account -a <ata>` prints the SPL Token Account struct to stdout.
    let output = ctx.run_metaboss(&["decode", "token-account", "-a", &ata_str]);
    assert_success(&output);

    let stdout = &output.stdout;
    // Verify it contains expected token account fields.
    assert!(
        stdout.contains("mint"),
        "decode token-account output should contain 'mint', got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("owner"),
        "decode token-account output should contain 'owner', got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("amount"),
        "decode token-account output should contain 'amount', got:\n{}",
        stdout
    );

    // The mint address and owner address should appear in the output.
    assert!(
        stdout.contains(&mint),
        "token account output should reference the mint address {}, got:\n{}",
        mint,
        stdout
    );
    assert!(
        stdout.contains(&ctx.keypair.pubkey().to_string()),
        "token account output should reference the owner address, got:\n{}",
        stdout
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Decode master edition
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_decode_master_edition() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("decode-master-edition");

    // Mint with --max-editions so that a master edition with max_supply is created.
    let mint = mint_test_nft_with_editions(&ctx, &temp_dir, 10)?;

    // `decode master -a <mint>` prints the master edition to stdout.
    let output = ctx.run_metaboss(&["decode", "master", "-a", &mint]);
    assert_success(&output);

    let stdout = &output.stdout;
    // The output should contain master edition fields.
    assert!(
        stdout.contains("supply"),
        "decode master output should contain 'supply', got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("max_supply"),
        "decode master output should contain 'max_supply', got:\n{}",
        stdout
    );

    // max_supply should be Some(10).
    assert!(
        stdout.contains("10"),
        "master edition max_supply should contain '10', got:\n{}",
        stdout
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Decode mint metadata with --full flag
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_decode_mint_full() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("decode-mint-full");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    let output_dir = temp_dir.join("decode_full_output");
    std::fs::create_dir_all(&output_dir)?;
    let output_dir_str = output_dir.to_string_lossy().to_string();

    // `decode mint -a <mint> --full --output <dir>` writes full metadata JSON.
    let output = ctx.run_metaboss(&[
        "decode",
        "mint",
        "-a",
        &mint,
        "--full",
        "--output",
        &output_dir_str,
    ]);
    assert_success(&output);

    // Verify the JSON file was created.
    let json_file = output_dir.join(format!("{}.json", mint));
    assert!(
        json_file.exists(),
        "decode mint --full should create {}.json in the output directory",
        mint
    );

    // Read and validate the JSON contains full metadata fields that are not
    // present in the non-full output (e.g., token_standard, primary_sale_happened).
    let json_content: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(&json_file)?)?;

    // Full metadata should include fields like primary_sale_happened, is_mutable, key.
    assert!(
        json_content.get("primary_sale_happened").is_some(),
        "full decode should contain 'primary_sale_happened', got:\n{}",
        serde_json::to_string_pretty(&json_content)?
    );
    assert!(
        json_content.get("is_mutable").is_some(),
        "full decode should contain 'is_mutable', got:\n{}",
        serde_json::to_string_pretty(&json_content)?
    );
    assert!(
        json_content.get("key").is_some(),
        "full decode should contain 'key', got:\n{}",
        serde_json::to_string_pretty(&json_content)?
    );

    // Basic fields should still be present.
    assert_eq!(
        json_content["name"].as_str().unwrap().trim_matches('\0'),
        "Test NFT",
        "name should match"
    );
    assert_eq!(
        json_content["symbol"].as_str().unwrap().trim_matches('\0'),
        "TNFT",
        "symbol should match"
    );

    // Also verify that a non-full decode does NOT contain these fields, to confirm
    // that --full actually changes the output.
    let non_full_dir = temp_dir.join("decode_nonfull_output");
    std::fs::create_dir_all(&non_full_dir)?;
    let non_full_dir_str = non_full_dir.to_string_lossy().to_string();

    let output = ctx.run_metaboss(&["decode", "mint", "-a", &mint, "--output", &non_full_dir_str]);
    assert_success(&output);

    let non_full_file = non_full_dir.join(format!("{}.json", mint));
    let non_full_content: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(&non_full_file)?)?;

    // NftData (non-full) should NOT have primary_sale_happened or is_mutable.
    assert!(
        non_full_content.get("primary_sale_happened").is_none(),
        "non-full decode should NOT contain 'primary_sale_happened'"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

// ---------------------------------------------------------------------------
// Test 5: Decode raw account bytes
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_decode_account_raw_bytes() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("decode-account-raw");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // `decode account <mint>` prints the raw bytes of the account to stdout.
    let output = ctx.run_metaboss(&["decode", "account", &mint]);
    assert_success(&output);

    let stdout = output.stdout.trim();
    // Output should be non-empty and look like a byte array (starts with '[').
    assert!(
        !stdout.is_empty(),
        "decode account output should not be empty"
    );
    assert!(
        stdout.starts_with('['),
        "decode account output should look like a byte array, got:\n{}",
        &stdout[..std::cmp::min(200, stdout.len())]
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}
