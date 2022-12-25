use std::fs::File;

use crate::{
    helius::HeliusResult,
    snapshot::{GetMintsArgs, Method},
    spinner::create_spinner,
};

use anyhow::Result;
use reqwest::Url;
use serde_json::json;

use super::Asset;

pub async fn get_mints(args: GetMintsArgs) -> Result<()> {
    let GetMintsArgs {
        address,
        method,
        api_key,
        output,
        indexer,
    } = args;

    let mut url = Url::parse("https://api.helius.xyz/v1/mintlist")?;
    url.set_query(Some(&format!("api-key={api_key}")));

    let mut assets: Vec<Asset> = Vec::new();
    let client = reqwest::Client::new();

    let mut pagination_token = None;

    let query = match method {
        Method::Creator => json!({
            "firstVerifiedCreators": [address.to_string()],
            "verifiedCollectionAddresses": []
        }
        ),
        Method::Collection => json!( {
            "firstVerifiedCreators": [],
            "verifiedCollectionAddresses": [address.to_string()]
        }
        ),
    };

    let spinner = create_spinner("Getting assets...");
    loop {
        let body = json!(
        {
            "query": query,
            "options": {
                "limit": 10000,
                "paginationToken": pagination_token
            }
        }
        );

        let response = client.post(url.clone()).json(&body).send().await?;
        let res: HeliusResult = response.json().await?;

        assets.extend(res.result);

        if res.pagination_token.is_empty() {
            break;
        }
        pagination_token = Some(res.pagination_token);
    }
    spinner.finish();

    let mut mints: Vec<String> = assets.iter().map(|asset| asset.mint.clone()).collect();
    mints.sort_unstable();

    let prefix = address[0..6].to_string();
    let f = File::create(format!("{output}/{prefix}_{method}_mints_{indexer}.json"))?;
    serde_json::to_writer_pretty(f, &mints)?;

    Ok(())
}
