mod common;

use std::io::Write;
use std::process::Command;

use anyhow::Result;
use regex::Regex;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use std::str::FromStr;

use common::{
    assert_success, decode_onchain_metadata, parse_mint_from_output, strip_debug_quotes, trim_null,
    TestContext,
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
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("mint-editions");

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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Mint an NFT with --immutable flag
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_one_with_immutable() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("mint-immutable");

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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Mint an NFT with --sign flag
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_one_with_sign() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("mint-sign");

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

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Create metadata for an existing bare SPL token mint
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_create_metadata() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("create-metadata");

    // Step 1: Create a bare SPL token mint using spl-token CLI.
    // This creates a mint with no Metaplex metadata account.
    // We explicitly set --mint-authority to the test keypair's pubkey so that
    // metaboss can sign the create-metadata transaction as the mint authority.
    let pubkey_str = ctx.keypair.pubkey().to_string();
    let create_token_output = Command::new("spl-token")
        .args([
            "create-token",
            "--url",
            &ctx.rpc_url,
            "--fee-payer",
            &ctx.keypair_path,
            "--mint-authority",
            &pubkey_str,
        ])
        .output()
        .expect("Failed to run spl-token create-token");

    assert!(
        create_token_output.status.success(),
        "spl-token create-token failed: {}",
        String::from_utf8_lossy(&create_token_output.stderr)
    );

    // Parse the mint address from spl-token output.
    // Output format: "Creating token <PUBKEY> under program ..."
    let stdout = String::from_utf8_lossy(&create_token_output.stdout);
    let mint_re = Regex::new(r"Creating token (\S+)").expect("invalid regex");
    let mint_address = mint_re
        .captures(&stdout)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .expect("Could not parse mint address from spl-token output");

    // Step 2: Write metadata JSON with name, symbol, uri fields.
    let metadata_json_path = temp_dir.join("metadata.json");
    {
        let metadata = serde_json::json!({
            "name": "Test Fungible",
            "symbol": "TFNG",
            "uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw"
        });
        let mut f = std::fs::File::create(&metadata_json_path)?;
        f.write_all(serde_json::to_string_pretty(&metadata)?.as_bytes())?;
    }
    let metadata_json_str = metadata_json_path.to_string_lossy().to_string();

    // Step 3: Run `metaboss create metadata` to attach metadata to the bare mint.
    let output = ctx.run_metaboss(&[
        "create",
        "metadata",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint_address,
        "-m",
        &metadata_json_str,
    ]);
    assert_success(&output);

    // Step 4: Verify the metadata was created on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint_address)?;
    assert_eq!(
        trim_null(&metadata.name),
        "Test Fungible",
        "metadata name should match"
    );
    assert_eq!(
        trim_null(&metadata.symbol),
        "TFNG",
        "metadata symbol should match"
    );
    assert!(
        trim_null(&metadata.uri).contains("arweave.net"),
        "metadata uri should contain the expected domain"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 5: Mint an asset via `mint asset` (Token Metadata unified handler)
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_asset() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("mint-asset");

    // Write an AssetData JSON file for a NonFungible asset.
    let creator_address = ctx.keypair.pubkey().to_string();
    let asset_data_path = temp_dir.join("asset_data.json");
    {
        let asset_data = serde_json::json!({
            "name": "Test Core Asset",
            "symbol": "TCA",
            "uri": "https://example.com/asset.json",
            "seller_fee_basis_points": 500,
            "creators": [
                {
                    "address": creator_address,
                    "verified": false,
                    "share": 100
                }
            ],
            "primary_sale_happened": false,
            "is_mutable": true,
            "token_standard": "NonFungible",
            "collection": null,
            "uses": null,
            "collection_details": null,
            "rule_set": null
        });
        let mut f = std::fs::File::create(&asset_data_path)?;
        f.write_all(serde_json::to_string_pretty(&asset_data)?.as_bytes())?;
    }
    let asset_data_str = asset_data_path.to_string_lossy().to_string();

    let output = ctx.run_metaboss(&[
        "mint",
        "asset",
        "-d",
        &asset_data_str,
        "-k",
        &ctx.keypair_path,
        "--max-print-edition-supply",
        "0",
    ]);
    assert_success(&output);

    // Parse the mint address from output: "Minted asset: <pubkey>" (Debug-formatted).
    let re = Regex::new(r"Minted asset: (\S+)").expect("invalid regex");
    let raw_mint = re
        .captures(&output.stdout)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .expect("Could not find 'Minted asset: <pubkey>' in output");
    let mint = strip_debug_quotes(&raw_mint);

    // Verify the asset exists on-chain by decoding its metadata.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        trim_null(&metadata.name),
        "Test Core Asset",
        "asset metadata name should match"
    );
    assert_eq!(
        trim_null(&metadata.symbol),
        "TCA",
        "asset metadata symbol should match"
    );
    assert!(
        trim_null(&metadata.uri).contains("example.com/asset.json"),
        "asset metadata uri should contain the expected URL"
    );

    // Verify the mint address is a valid pubkey.
    assert!(
        Pubkey::from_str(&mint).is_ok(),
        "minted asset address should be a valid pubkey"
    );

    Ok(())
}
