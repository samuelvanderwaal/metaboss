use super::*;

pub fn update_rule_set_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint: &str,
    new_rule_set: &str,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let md = decode_metadata_from_mint(client, mint)?;

    // We need the token account passed in for pNFT updates.
    let token = Some(get_nft_token_account(client, mint)?);

    let mint = Pubkey::from_str(mint)?;
    let new_rule_set = Pubkey::from_str(new_rule_set)?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the rule set.
    let UpdateArgs::V1 {
        ref mut rule_set, ..
    } = update_args;

    *rule_set = RuleSetToggle::Set(new_rule_set);

    let current_rule_set = if let Some(ProgrammableConfig::V1 { rule_set }) = md.programmable_config
    {
        rule_set
    } else {
        None
    };

    // Metaboss UpdateAssetArgs enum.
    let args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &keypair,
        mint,
        token,
        delegate_record: None, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    let update_result = update_asset(client, args)?;

    println!("Updated asset: {mint:?}");
    println!("Update signature: {update_result:?}");

    Ok(())
}

pub fn clear_rule_set_one(
    client: &RpcClient,
    keypair_path: Option<String>,
    mint: &str,
) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(keypair_path, solana_opts);

    let md = decode_metadata_from_mint(client, mint)?;

    // We need the token account passed in for pNFT updates.
    let token = Some(get_nft_token_account(client, mint)?);

    let mint = Pubkey::from_str(mint)?;

    // Add metadata delegate record here later.

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    // Update the rule set.
    let UpdateArgs::V1 {
        ref mut rule_set, ..
    } = update_args;

    *rule_set = RuleSetToggle::Clear;

    let current_rule_set = if let Some(ProgrammableConfig::V1 { rule_set }) = md.programmable_config
    {
        rule_set
    } else {
        None
    };

    // Metaboss UpdateAssetArgs enum.
    let args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &keypair,
        mint,
        token,
        delegate_record: None, // Not supported yet in update.
        current_rule_set,
        update_args,
    };

    let update_result = update_asset(client, args)?;

    println!("Updated asset: {mint:?}");
    println!("Update signature: {update_result:?}");

    Ok(())
}
