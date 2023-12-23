use core::panic;
use std::path::PathBuf;

use anyhow::Result;
use solana_program::pubkey::Pubkey;

use super::ClientLike;

pub struct FcvaArgs {
    pub client: ClientLike,
    pub creator: Pubkey,
    pub output: PathBuf,
}

pub fn fcva_mints(args: FcvaArgs) -> Result<()> {
    let client = match args.client {
        ClientLike::RpcClient(client) => panic!("Not supported for this method"),
        ClientLike::DasClient(client) => client,
    };

    // let mut url = Url::parse("https://api.helius.xyz/v1/mintlist")?;
    // url.set_query(Some(&format!("api-key={api_key}")));

    // let mut assets: Vec<Asset> = Vec::new();
    // let client = reqwest::Client::new();

    // let mut pagination_token = None;

    // let query = match method {
    //     Method::Creator => json!({
    //         "firstVerifiedCreators": [address.to_string()],
    //         "verifiedCollectionAddresses": []
    //     }
    //     ),
    //     Method::Collection => json!( {
    //         "firstVerifiedCreators": [],
    //         "verifiedCollectionAddresses": [address.to_string()]
    //     }
    //     ),
    // };

    // let spinner = create_spinner("Getting assets...");
    // loop {
    //     let body = json!(
    //     {
    //         "query": query,
    //         "options": {
    //             "limit": 10000,
    //             "paginationToken": pagination_token
    //         }
    //     }
    //     );

    //     let response = client.post(url.clone()).json(&body).send().await?;
    //     let res: HeliusResult = response.json().await?;

    //     assets.extend(res.result);

    //     if res.pagination_token.is_empty() {
    //         break;
    //     }
    //     pagination_token = Some(res.pagination_token);
    // }
    // spinner.finish();

    // let mut mints: Vec<String> = assets.iter().map(|asset| asset.mint.clone()).collect();

    // Ok(mints)

    Ok(())
}
