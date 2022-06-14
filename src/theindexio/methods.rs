use crate::spinner::create_alt_spinner;
use anyhow::Result;

use super::data::*;

use serde_json::json;

pub async fn get_verified_creator_accounts(creator: &str) -> Result<Vec<GPAResult>> {
    let method = "getProgramAccounts";
    let url = "https://rpc.theindex.io";
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

pub async fn get_holder_token_accounts(mint_account: String) -> Result<Vec<GPAResult>> {
    let method = "getProgramAccounts";
    let url = "https://rpc.theindex.io";
    let params = json!(
    [
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
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

    let spinner = create_alt_spinner("Fetching data from TheIndex.io. . .");
    let response = client.post(url).json(&jrpc).send().await?;
    spinner.finish();

    let res: GPAResponse = response.json().await?;

    Ok(res.result)
}
