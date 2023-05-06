use metaboss_lib::decode::decode_rule_set;

use super::*;

pub fn process_decode_rule_set(
    client: &RpcClient,
    rule_set_pubkey: Pubkey,
    revision: Option<usize>,
) -> AnyResult<()> {
    let rule_set = decode_rule_set(client, &rule_set_pubkey, revision)?;

    let f = File::create(format!("{rule_set_pubkey}_rule_set.json"))?;

    serde_json::to_writer_pretty(f, &rule_set)?;

    Ok(())
}
