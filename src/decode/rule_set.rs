use super::*;

pub fn process_decode_rule_set(
    _client: &RpcClient,
    _rule_set_pubkey: Pubkey,
    _revision: Option<usize>,
) -> AnyResult<()> {
    Err(anyhow!(
        "Rule set decoding is no longer supported. Token Auth Rules have been deprecated."
    ))
}
