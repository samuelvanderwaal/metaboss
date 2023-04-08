use anyhow::Result;
use mpl_token_metadata::{
    instruction::UsesToggle,
    state::{UseMethod, Uses},
};
use solana_client::rpc_client::RpcClient;

use super::*;

pub struct UsesArgs {
    pub client: RpcClient,
    pub keypair: Option<String>,
    pub account: String,
    pub method: String,
    pub remaining: u64,
    pub total: u64,
    pub overwrite: bool,
}

pub fn update_uses_one(args: UsesArgs) -> Result<Signature, ActionError> {
    let solana_opts = parse_solana_config();
    let keypair = parse_keypair(args.keypair, solana_opts);

    let current_md = decode_metadata_from_mint(&args.client, args.account.clone())
        .map_err(|e| ActionError::ActionFailed(args.account.to_string(), e.to_string()))?;

    let use_method = match args.method.to_lowercase().as_str() {
        "burn" => UseMethod::Burn,
        "multiple" => UseMethod::Multiple,
        "single" => UseMethod::Single,
        _ => {
            return Err(ActionError::ActionFailed(
                args.account.to_string(),
                "Invalid Uses method. Must be one of: burn, multiple, single".to_string(),
            ));
        }
    };

    let new_uses = Uses {
        use_method,
        remaining: args.remaining,
        total: args.total,
    };

    // Only overwrite existing uses if the override flag is set
    if current_md.uses.is_some() && !args.overwrite {
        return Err(ActionError::ActionFailed(
            args.account,
            "Uses already exist for this token. Use the --overwrite flag to overwrite.".to_string(),
        ));
    }

    // Token Metadata UpdateArgs enum.
    let mut update_args = UpdateArgs::default();

    let UpdateArgs::V1 { ref mut uses, .. } = update_args;
    *uses = UsesToggle::Set(new_uses);

    // Metaboss UpdateAssetArgs enum.
    let update_args = UpdateAssetArgs::V1 {
        payer: None,
        authority: &keypair,
        mint: args.account.clone(),
        token: None::<String>,
        delegate_record: None::<String>, // Not supported yet in update.
        update_args,
    };

    update_asset(&args.client, update_args)
        .map_err(|e| ActionError::ActionFailed(args.account.to_string(), e.to_string()))
}
