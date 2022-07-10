use super::common::*;
use crate::collections::data::*;
use crate::derive::derive_metadata_pda;
use crate::spinner::create_alt_spinner;
use crate::theindexio::THE_INDEX_MAINNET;
use borsh::BorshDeserialize;

pub async fn get_collection_items(
    collection_mint: String,
    method: GetCollectionItemsMethods,
    api_key: Option<String>,
) -> AnyResult<()> {
    match method {
        GetCollectionItemsMethods::TheIndexIO => {
            if let Some(key) = api_key {
                get_collection_items_by_the_index_io(collection_mint, key).await?
            } else {
                return Err(anyhow!(
                    "This method requires an index key for TheIndex.io."
                ));
            }
        }
    }
    Ok(())
}

pub async fn get_collection_items_by_the_index_io(
    collection_mint: String,
    api_key: String,
) -> AnyResult<()> {
    let jrpc = JRPCRequest::new("getNFTsByCollection", vec![collection_mint.clone()]);
    let url = format!("{THE_INDEX_MAINNET}/{api_key}");
    let client = reqwest::Client::new();

    let spinner = create_alt_spinner("Fetching data from TheIndex.io. . .");
    let response = client.post(url).json(&jrpc).send().await?;
    spinner.finish();

    let res: RpcResponse = response.json().await?;

    let mints: Vec<String> = res
        .result
        .iter()
        .map(|nft| nft.metadata.mint.clone())
        .collect();

    let file_name = format!("{collection_mint}_collection_items.json");
    let f = File::create(&file_name).unwrap();
    serde_json::to_writer_pretty(f, &mints).unwrap();
    println!("Data written to {file_name}");

    Ok(())
}

pub async fn check_collection_items(
    async_client: AsyncRpcClient,
    collection_mint: String,
    mint_list_path: String,
    debug: bool,
) -> AnyResult<()> {
    let f = File::open(mint_list_path)?;
    let mut mint_list: Vec<String> = serde_json::from_reader(f)?;
    let mint_list_length = mint_list.len();

    let mut collections: HashMap<String, Vec<String>> = HashMap::new();
    let mut handles = Vec::new();
    let mut errors = Vec::new();

    let client = Arc::new(async_client);

    let spinner = create_alt_spinner("Sending network requests and awaiting results...");
    for mint in mint_list.drain(0..cmp::min(mint_list.len(), PARALLEL_LIMIT)) {
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            get_mint_collection(&client, mint.to_string()).await
        }));
    }

    while !handles.is_empty() {
        match select_all(handles).await {
            (Ok(res), _index, remaining) => {
                handles = remaining;

                if res.is_ok() {
                    let (mint, collection_opt) = res.unwrap();
                    match collection_opt {
                        Some(collection) => {
                            collections
                                .entry(collection.key.to_string())
                                .or_insert_with(Vec::new)
                                .push(mint.to_string());
                        }
                        None => {
                            collections
                                .entry("none".to_string())
                                .or_insert_with(Vec::new)
                                .push(mint.to_string());
                        }
                    }
                } else {
                    errors.push(res.err().unwrap());
                }
            }
            (Err(err), _index, remaining) => {
                errors.push(err.into());
                // ignoring all errors
                handles = remaining;
            }
        }

        if !mint_list.is_empty() {
            // if we are half way through, let spawn more transactions
            if (PARALLEL_LIMIT - handles.len()) > (PARALLEL_LIMIT / 2) {
                // syncs cache (checkpoint)

                for mint in mint_list.drain(0..cmp::min(mint_list.len(), PARALLEL_LIMIT)) {
                    let client = client.clone();
                    handles.push(tokio::spawn(async move {
                        get_mint_collection(&client, mint.to_string()).await
                    }));
                }
            }
        }
    }
    spinner.finish();

    let mint_items = collections.get(&collection_mint).ok_or_else(|| {
        anyhow!("No mints found for this parent. Run with --debug to see more details.")
    })?;
    let keys: Vec<&String> = collections.keys().collect();

    // Debug mode writes a JSON file containing all items and which collection parents they belong to.
    if debug {
        println!("Writing debug file...");
        let out = File::create(format!("{collection_mint}-debug-collections.json"))?;
        serde_json::to_writer(out, &collections)?;
    }

    // Check if there's the only one and correct collection parent associated with the mint list and that all items in the list belong to it.
    if !keys.contains(&&collection_mint) || keys.len() != 1 || mint_items.len() != mint_list_length
    {
        return Err(anyhow!("Not all mints from the list belong to this parent. Run with --debug to see more details."));
    }

    println!("All mints in are the collection!");
    Ok(())
}

async fn get_mint_collection<'a>(
    client: &AsyncRpcClient,
    mint: String,
) -> AnyResult<(String, Option<MdCollection>)> {
    let mint_pubkey = Pubkey::from_str(&mint)?;
    let metadata_pubkey = derive_metadata_pda(&mint_pubkey);
    let data = client.get_account_data(&metadata_pubkey).await?;
    let md = Metadata::deserialize(&mut data.as_slice())?;

    Ok((mint, md.collection))
}
