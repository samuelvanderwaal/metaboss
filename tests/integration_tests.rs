mod common;

use std::str::FromStr;

use anyhow::Result;
use metaboss_lib::derive::derive_metadata_pda;
use solana_program::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;

use common::{assert_success, decode_onchain_metadata, mint_test_nft, trim_null, TestContext};

// ---------------------------------------------------------------------------
// Test 1: Mint an NFT and verify its on-chain metadata via decode
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_one_and_decode() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("decode");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Also exercise the CLI decode command to ensure it writes the JSON file.
    let decode_output_dir = temp_dir.join("decode_output");
    std::fs::create_dir_all(&decode_output_dir)?;
    let decode_output_str = decode_output_dir.to_string_lossy().to_string();

    let output = ctx.run_metaboss(&[
        "decode",
        "mint",
        "-a",
        &mint,
        "--output",
        &decode_output_str,
    ]);
    assert_success(&output);

    // Verify the JSON file was created by the CLI.
    let json_file = decode_output_dir.join(format!("{}.json", mint));
    assert!(
        json_file.exists(),
        "decode mint should create {}.json in the output directory",
        mint
    );

    // Read and validate the JSON contents.
    let json_content: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(&json_file)?)?;
    assert_eq!(
        json_content["name"].as_str().unwrap(),
        "Test NFT",
        "name should match"
    );
    assert_eq!(
        json_content["symbol"].as_str().unwrap(),
        "TNFT",
        "symbol should match"
    );
    assert_eq!(
        json_content["seller_fee_basis_points"].as_u64().unwrap(),
        100,
        "seller_fee_basis_points should be 100"
    );

    // Verify on-chain metadata via the library.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(trim_null(&metadata.name), "Test NFT");
    assert_eq!(trim_null(&metadata.symbol), "TNFT");
    assert_eq!(
        trim_null(&metadata.uri),
        "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw"
    );
    assert_eq!(metadata.seller_fee_basis_points, 100);

    // Verify creators include the test keypair.
    let creators = metadata.creators.expect("creators should be present");
    let creator_pubkey = ctx.keypair.pubkey().to_string();
    assert!(
        creators
            .iter()
            .any(|c| c.address.to_string() == creator_pubkey),
        "creators should contain the test keypair address"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Mint an NFT, update its URI and name, and verify changes
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_update_uri_and_name() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("update");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Update URI.
    let output = ctx.run_metaboss(&[
        "update",
        "uri",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-uri",
        "https://example.com/updated",
    ]);
    assert_success(&output);

    // Verify URI was updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        trim_null(&metadata.uri),
        "https://example.com/updated",
        "URI should be updated"
    );

    // Update name.
    let output = ctx.run_metaboss(&[
        "update",
        "name",
        "-k",
        &ctx.keypair_path,
        "-a",
        &mint,
        "--new-name",
        "Updated NFT",
    ]);
    assert_success(&output);

    // Verify name was updated on-chain.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    assert_eq!(
        trim_null(&metadata.name),
        "Updated NFT",
        "Name should be updated"
    );

    // URI should still be the updated value.
    assert_eq!(
        trim_null(&metadata.uri),
        "https://example.com/updated",
        "URI should remain updated after name change"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Mint an NFT and burn it
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_and_burn() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("burn");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Burn the NFT using `burn-nft one`.
    let output = ctx.run_metaboss(&["burn-nft", "one", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // After burning, the token account should no longer exist or have 0 balance.
    let mint_pubkey = Pubkey::from_str(&mint)?;
    let ata = get_associated_token_address(&ctx.keypair.pubkey(), &mint_pubkey);
    let account_result = ctx.client.get_account(&ata);

    // The account should either be gone or have zero balance.
    match account_result {
        Err(_) => {
            // Account no longer exists -- expected after burn.
        }
        Ok(account) => {
            let token_account = TokenAccount::unpack(&account.data)
                .expect("should be able to unpack token account if it still exists");
            assert_eq!(
                token_account.amount, 0,
                "token account balance should be 0 after burn"
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 4: Mint an NFT and transfer it to another wallet
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_and_transfer() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("transfer");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Generate a receiver keypair (no need to fund it; the sender pays).
    let receiver = Keypair::new();
    let receiver_pubkey_str = receiver.pubkey().to_string();

    // Transfer the NFT.
    let output = ctx.run_metaboss(&[
        "transfer",
        "asset",
        "-k",
        &ctx.keypair_path,
        "-R",
        &receiver_pubkey_str,
        "--mint",
        &mint,
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
        "receiver should hold exactly 1 token after transfer"
    );

    // Verify the sender no longer holds the token.
    let sender_ata = get_associated_token_address(&ctx.keypair.pubkey(), &mint_pubkey);
    let sender_account = ctx.client.get_account(&sender_ata);
    match sender_account {
        Err(_) => {
            // Account closed -- expected.
        }
        Ok(account) => {
            let token_account =
                TokenAccount::unpack(&account.data).expect("should unpack sender token account");
            assert_eq!(
                token_account.amount, 0,
                "sender should hold 0 tokens after transfer"
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 5: Mint an NFT and sign it (verify creator)
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_mint_and_sign() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("sign");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Before signing, the creator should be unverified.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    let creators = metadata.creators.expect("creators should be present");
    let creator = creators
        .iter()
        .find(|c| c.address == ctx.keypair.pubkey())
        .expect("test keypair should be listed as a creator");
    assert!(
        !creator.verified,
        "creator should be unverified before signing"
    );

    // Sign the metadata.
    let output = ctx.run_metaboss(&["sign", "one", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // After signing, the creator should be verified.
    let metadata = decode_onchain_metadata(&ctx, &mint)?;
    let creators = metadata.creators.expect("creators should be present");
    let creator = creators
        .iter()
        .find(|c| c.address == ctx.keypair.pubkey())
        .expect("test keypair should be listed as a creator");
    assert!(creator.verified, "creator should be verified after signing");

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 6: Mint an NFT and burn it using `burn asset`
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_burn_asset() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("burn-asset");
    let mint = mint_test_nft(&ctx, &temp_dir)?;

    // Burn the NFT using `burn asset`.
    let output = ctx.run_metaboss(&["burn", "asset", "-k", &ctx.keypair_path, "-a", &mint]);
    assert_success(&output);

    // After burning, the token account should no longer exist or have 0 balance.
    let mint_pubkey = Pubkey::from_str(&mint)?;
    let ata = get_associated_token_address(&ctx.keypair.pubkey(), &mint_pubkey);
    let account_result = ctx.client.get_account(&ata);

    match account_result {
        Err(_) => {
            // Account no longer exists -- expected after burn.
        }
        Ok(account) => {
            let token_account = TokenAccount::unpack(&account.data)
                .expect("should be able to unpack token account if it still exists");
            assert_eq!(
                token_account.amount, 0,
                "token account balance should be 0 after burn"
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 7: Derive metadata PDA and verify it matches the expected value
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_derive_metadata_pda() -> Result<()> {
    let mut ctx = TestContext::new()?;
    let temp_dir = ctx.create_temp_dir("derive");
    let mint = mint_test_nft(&ctx, &temp_dir)?;
    let mint_pubkey = Pubkey::from_str(&mint)?;

    // Compute the expected metadata PDA using the library.
    let expected_pda = derive_metadata_pda(&mint_pubkey);

    // Run the CLI derive command.
    let output = ctx.run_metaboss(&["derive", "metadata", &mint]);
    assert_success(&output);

    // The output should contain the PDA pubkey.
    let stdout = output.stdout.trim();
    assert!(
        stdout.contains(&expected_pda.to_string()),
        "derive metadata output should contain the expected PDA.\nExpected: {}\nGot: {}",
        expected_pda,
        stdout
    );

    Ok(())
}
