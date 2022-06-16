use crate::spinner::create_alt_spinner;
use anyhow::Result;
use spl_token::ID as SPL_TOKEN_ID;

use super::data::*;

use serde_json::json;

pub async fn get_verified_creator_accounts(
    api_key: String,
    creator: &str,
) -> Result<Vec<GPAResult>> {
    let method = "getProgramAccounts";
    let url = format!("{THE_INDEX_MAINNET}/{api_key}");
    let params = json!(
    [
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        {
            "commitment": "finalized",
            "encoding": "base64",
            "filters": [
                {
                    "memcmp": {
                        "offset": 326u32,
                        "bytes": creator
                    }
                },
                {
                    "memcmp": {
                        "offset": 358u32,
                        "bytes": "2"
                    }
                }
                ]
            }
            ]);

    let jrpc = JRPCRequest::new(method, params);

    let client = reqwest::Client::new();

    let spinner = create_alt_spinner("Fetching data from TheIndex.io. . .");
    let response = client.post(url).json(&jrpc).send().await?;
    spinner.finish();

    let res: GPAResponse = response.json().await?;

    Ok(res.result)
}

pub async fn get_holder_token_accounts(
    api_key: &String,
    mint_account: &str,
) -> Result<Vec<GPAResult>> {
    let method = "getProgramAccounts";
    let url = format!("{THE_INDEX_MAINNET}/{api_key}");
    let params = json!(
    [
        SPL_TOKEN_ID.to_string(),
        {
            "commitment": "finalized",
            "encoding": "base64",
            "filters": [
                {
                    "memcmp": {
                        "offset": 0,
                        "bytes": mint_account
                    }
                },
                {
                    "dataSize": 165
                }
                ]
            }
            ]);

    let jrpc = JRPCRequest::new(method, params);

    let client = reqwest::Client::new();

    let response = client.post(url).json(&jrpc).send().await?;
    let res: GPAResponse = response.json().await?;

    Ok(res.result)
}

pub async fn get_token_largest_accounts(mint_account: String) -> Result<TLAResult> {
    let method = "getTokenLargestAccounts";
    let url = "https://rpc.theindex.io";
    let params = json!([mint_account]);

    let jrpc = JRPCRequest::new(method, params);

    let client = reqwest::Client::new();

    let response = client.post(url).json(&jrpc).send().await?;
    let res: TLAResult = response.json().await?;

    Ok(res)
}
