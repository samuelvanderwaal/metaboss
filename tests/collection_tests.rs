mod common;

use anyhow::Result;

use common::{
    assert_success, create_temp_dir, decode_onchain_metadata, mint_test_nft,
    parse_mint_from_output, strip_debug_quotes, TestContext,
};

/// Helper: mint a sized collection parent NFT using `mint one --sized`.
fn mint_collection_nft(ctx: &TestContext, temp_dir: &std::path::Path) -> Result<String> {
    let nft_json = temp_dir.join("collection_nft.json");
    ctx.create_test_nft_json(&nft_json)?;

    let nft_json_str = nft_json.to_string_lossy().to_string();
    let output = ctx.run_metaboss(&[
        "mint",
        "one",
        "-d",
        &nft_json_str,
        "-k",
        &ctx.keypair_path,
        "--sized",
        "--max-editions",
        "0",
    ]);
    assert_success(&output);

    let raw_mint = parse_mint_from_output(&output.stdout);
    Ok(strip_debug_quotes(&raw_mint))
}

// ---------------------------------------------------------------------------
// Test 1: Mint a collection NFT and a child NFT, set and verify collection
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_collections_set_and_verify() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("set-and-verify");

    // Mint the collection parent NFT (sized).
    let collection_mint = mint_collection_nft(&ctx, &temp_dir)?;

    // Mint a child NFT.
    let child_mint = mint_test_nft(&ctx, &temp_dir)?;

    // Before setting collection, the child should have no collection field.
    let metadata = decode_onchain_metadata(&ctx, &child_mint)?;
    assert!(
        metadata.collection.is_none(),
        "child NFT should have no collection before set-and-verify"
    );

    // Run `collections set-and-verify`.
    let output = ctx.run_metaboss(&[
        "collections",
        "set-and-verify",
        "-k",
        &ctx.keypair_path,
        "--collection-mint",
        &collection_mint,
        "--nft-mint",
        &child_mint,
    ]);
    assert_success(&output);

    // Decode the child NFT and verify the collection is set and verified.
    let metadata = decode_onchain_metadata(&ctx, &child_mint)?;
    let collection = metadata
        .collection
        .expect("child NFT should have a collection after set-and-verify");
    assert_eq!(
        collection.key.to_string(),
        collection_mint,
        "collection key should match the collection mint"
    );
    assert!(
        collection.verified,
        "collection should be verified after set-and-verify"
    );

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 2: Full lifecycle: set-and-verify, unverify, verify
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_collections_verify_and_unverify() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("verify-unverify");

    // Mint collection parent and child NFTs.
    let collection_mint = mint_collection_nft(&ctx, &temp_dir)?;
    let child_mint = mint_test_nft(&ctx, &temp_dir)?;

    // Set and verify the collection on the child.
    let output = ctx.run_metaboss(&[
        "collections",
        "set-and-verify",
        "-k",
        &ctx.keypair_path,
        "--collection-mint",
        &collection_mint,
        "--nft-mint",
        &child_mint,
    ]);
    assert_success(&output);

    // Confirm it is verified.
    let metadata = decode_onchain_metadata(&ctx, &child_mint)?;
    let collection = metadata
        .collection
        .expect("child should have collection after set-and-verify");
    assert!(
        collection.verified,
        "collection should be verified initially"
    );

    // Unverify the collection.
    let output = ctx.run_metaboss(&[
        "collections",
        "unverify",
        "-k",
        &ctx.keypair_path,
        "--collection-mint",
        &collection_mint,
        "--nft-mint",
        &child_mint,
    ]);
    assert_success(&output);

    // Verify collection.verified is now false.
    let metadata = decode_onchain_metadata(&ctx, &child_mint)?;
    let collection = metadata
        .collection
        .expect("child should still have collection after unverify");
    assert_eq!(
        collection.key.to_string(),
        collection_mint,
        "collection key should still match after unverify"
    );
    assert!(
        !collection.verified,
        "collection should be unverified after unverify"
    );

    // Re-verify the collection.
    let output = ctx.run_metaboss(&[
        "collections",
        "verify",
        "-k",
        &ctx.keypair_path,
        "--collection-mint",
        &collection_mint,
        "--nft-mint",
        &child_mint,
    ]);
    assert_success(&output);

    // Confirm it is verified again.
    let metadata = decode_onchain_metadata(&ctx, &child_mint)?;
    let collection = metadata
        .collection
        .expect("child should still have collection after re-verify");
    assert!(
        collection.verified,
        "collection should be verified after re-verify"
    );

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

// ---------------------------------------------------------------------------
// Test 3: Set collection size on an unsized collection NFT
// ---------------------------------------------------------------------------
#[test]
#[ignore = "requires solana-test-validator (run with --ignored)"]
fn test_collections_set_size() -> Result<()> {
    let ctx = TestContext::new()?;
    let temp_dir = create_temp_dir("set-size");

    // Mint an unsized collection NFT (no --sized flag) with a master edition.
    let nft_json = temp_dir.join("collection_nft.json");
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
        "0",
    ]);
    assert_success(&output);
    let collection_mint = strip_debug_quotes(&parse_mint_from_output(&output.stdout));

    // Verify it has no collection_details (unsized).
    let metadata = decode_onchain_metadata(&ctx, &collection_mint)?;
    assert!(
        metadata.collection_details.is_none(),
        "unsized collection should have no collection_details"
    );

    // Set collection size to 100 (converts unsized → sized).
    let output = ctx.run_metaboss(&[
        "collections",
        "set-size",
        "-k",
        &ctx.keypair_path,
        "--collection-mint",
        &collection_mint,
        "--size",
        "100",
    ]);
    assert_success(&output);

    // Decode and check that collection_details now reflects the size.
    let metadata = decode_onchain_metadata(&ctx, &collection_mint)?;
    match &metadata.collection_details {
        Some(mpl_token_metadata::types::CollectionDetails::V1 { size }) => {
            assert_eq!(*size, 100, "collection size should be 100 after set-size");
        }
        other => {
            panic!(
                "expected CollectionDetails::V1 with size 100, got {:?}",
                other
            );
        }
    }

    // Clean up temp dir.
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}
