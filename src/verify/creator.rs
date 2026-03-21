use metaboss_lib::{data::Asset, decode::ToPubkey, transaction::send_and_confirm_tx};
use mpl_token_metadata::{instructions::VerifyCreatorV1Builder, types::TokenStandard};
use solana_sdk::{compute_budget::ComputeBudgetInstruction, signer::Signer};

use super::*;

pub struct VerifyCreatorArgs {
    pub client: Arc<RpcClient>,
    pub keypair: Arc<Keypair>,
    pub mint: String,
    pub priority: Priority,
}

pub struct VerifyCreatorAllArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub mint_list: Option<String>,
    pub cache_file: Option<String>,
    pub rate_limit: usize,
    pub retries: u8,
    pub priority: Priority,
}

pub async fn verify_creator(args: VerifyCreatorArgs) -> Result<Signature, ActionError> {
    let mint = args
        .mint
        .clone()
        .to_pubkey()
        .map_err(|e| ActionError::ActionFailed(args.mint.clone(), e.to_string()))?;
    let asset = Asset::new(mint);

    let md = asset
        .get_metadata(&args.client)
        .map_err(|e| ActionError::ActionFailed(args.mint.clone(), e.to_string()))?;

    if !matches!(
        md.token_standard,
        Some(TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible) | None
    ) {
        return Err(ActionError::ActionFailed(
            args.mint.clone(),
            "Only NFTs or pNFTs can have creators be verified".to_string(),
        ));
    }

    let mut verify_builder = VerifyCreatorV1Builder::new();
    verify_builder
        .authority(args.keypair.pubkey())
        .metadata(asset.metadata);

    let verify_ix = verify_builder.instruction();

    let micro_lamports = match args.priority {
        Priority::None => 20,
        Priority::Low => 20_000,
        Priority::Medium => 200_000,
        Priority::High => 1_000_000,
        Priority::Max => 2_000_000,
    };

    let instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_price(micro_lamports),
        verify_ix,
    ];

    send_and_confirm_tx(&args.client, &[&args.keypair], &instructions)
        .map_err(|e| ActionError::ActionFailed(args.mint.to_string(), e.to_string()))
}

pub struct VerifyCreatorAll {}

#[async_trait]
impl Action for VerifyCreatorAll {
    fn name() -> &'static str {
        "verify-creator-all"
    }

    async fn action(args: RunActionArgs) -> Result<(), ActionError> {
        verify_creator(VerifyCreatorArgs {
            client: args.client.clone(),
            keypair: args.keypair.clone(),
            mint: args.mint_account.clone(),
            priority: args.priority.clone(),
        })
        .await
        .map(|_| ())
    }
}

pub async fn verify_creator_all(args: VerifyCreatorAllArgs) -> AnyResult<()> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let mint_list = parse_mint_list(args.mint_list, &args.cache_file)?;

    // We don't support an optional payer for this action currently.
    let payer = None;

    let args = BatchActionArgs {
        client: args.client,
        keypair,
        payer,
        mint_list,
        cache_file: args.cache_file,
        new_value: NewValue::None,
        rate_limit: args.rate_limit,
        retries: args.retries,
        priority: args.priority,
    };
    VerifyCreatorAll::run(args).await
}
