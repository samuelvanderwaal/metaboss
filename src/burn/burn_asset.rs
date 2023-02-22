use mpl_token_metadata::pda::find_token_record_account;

use super::*;

pub struct BurnAssetArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint_account: String,
    pub token_account: Option<String>,
    pub amount: u64,
}

pub struct BurnAssetAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub batch_size: usize,
    pub retries: u8,
}

pub async fn burn_asset(args: BurnAssetArgs) -> Result<Signature, ActionError> {
    let current_md = decode_metadata_from_mint(&args.client, args.mint_account.clone())
        .map_err(|e| ActionError::ActionFailed(args.mint_account.clone(), e.to_string()))?;

    let mint = Pubkey::from_str(&args.mint_account)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?;

    let token = if let Some(token) = args.token_account {
        Pubkey::from_str(&token)
            .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))?
    } else {
        get_associated_token_address(&args.keypair.pubkey(), &mint)
    };

    let token_record =
        if let Some(TokenStandard::ProgrammableNonFungible) = current_md.token_standard {
            let (token_record, _) = find_token_record_account(&mint, &token);

            Some(token_record)
        } else {
            None
        };

    let burn_args = metaboss_lib::burn::BurnAssetArgs::V1 {
        authority: &args.keypair,
        mint,
        token,
        token_record,
        amount: args.amount,
    };

    metaboss_lib::burn::burn_asset(&args.client, burn_args)
        .map_err(|e| ActionError::ActionFailed(args.mint_account.to_string(), e.to_string()))
}
