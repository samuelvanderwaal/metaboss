use anyhow::{anyhow, Result};
use metaplex_token_metadata::state::Metadata;
use metaplex_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
use serde::Serialize;
use solana_account_decoder::{
    parse_account_data::{parse_account_data, AccountAdditionalData, ParsedAccount},
    UiAccountEncoding,
};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::{
    account::Account,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
};
use spl_token::ID as TOKEN_PROGRAM_ID;
use std::{fs::File, str::FromStr};

use crate::constants::*;

#[derive(Debug, Serialize, Clone)]
struct Holder {
    owner_wallet: String,
    associated_token_address: String,
    mint_account: String,
    metadata_account: String,
}

#[derive(Debug, Serialize)]
struct CandyMachineProgramAccounts {
    config_accounts: Vec<ConfigAccount>,
    candy_machine_accounts: Vec<CandyMachineAccount>,
}

#[derive(Debug, Serialize)]
struct ConfigAccount {
    address: String,
    data_len: usize,
}

#[derive(Debug, Serialize)]
struct CandyMachineAccount {
    address: String,
    data_len: usize,
}

pub fn get_mints(
    client: &RpcClient,
    update_authority: &Option<String>,
    candy_machine_id: &Option<String>,
    output: &String,
) -> Result<()> {
    let accounts = if let Some(update_authority) = update_authority {
        get_mints_by_update_authority(client, update_authority)?
    } else if let Some(candy_machine_id) = candy_machine_id {
        get_cm_creator_accounts(client, candy_machine_id)?
    } else {
        return Err(anyhow!(
            "Must specify either --update-authority or --candy-machine-id"
        ));
    };

    let mut mint_accounts: Vec<String> = Vec::new();

    for (_, account) in accounts {
        let metadata: Metadata = try_from_slice_unchecked(&account.data)?;

        mint_accounts.push(metadata.mint.to_string());
    }

    let prefix = if let Some(update_authority) = update_authority {
        update_authority
    } else if let Some(candy_machine_id) = candy_machine_id {
        candy_machine_id
    } else {
        return Err(anyhow!(
            "Must specify either --update-authority or --candy-machine-id"
        ));
    };

    let mut file = File::create(format!("{}/{}_mint_accounts.json", output, prefix))?;
    serde_json::to_writer(&mut file, &mint_accounts)?;

    Ok(())
}

pub fn snapshot_holders(
    client: &RpcClient,
    update_authority: &Option<String>,
    candy_machine_id: &Option<String>,
    output: &String,
) -> Result<()> {
    let accounts = if let Some(update_authority) = update_authority {
        get_mints_by_update_authority(client, update_authority)?
    } else if let Some(candy_machine_id) = candy_machine_id {
        get_cm_creator_accounts(client, candy_machine_id)?
    } else {
        return Err(anyhow!(
            "Must specify either --update-authority or --candy-machine-id"
        ));
    };

    let mut nft_holders: Vec<Holder> = Vec::new();

    for (metadata_pubkey, account) in accounts {
        let metadata: Metadata = try_from_slice_unchecked(&account.data)?;

        let token_accounts = get_holder_token_accounts(client, metadata.mint.to_string())?;

        for (associated_token_address, account) in token_accounts {
            let data = parse_account_data(
                &metadata.mint,
                &TOKEN_PROGRAM_ID,
                &account.data,
                Some(AccountAdditionalData {
                    spl_token_decimals: Some(0),
                }),
            )?;
            let amount = parse_token_amount(&data)?;

            // Only include current holder of the NFT.
            if amount == 1 {
                let owner_wallet = parse_owner(&data)?;
                let associated_token_address = associated_token_address.to_string();
                let holder = Holder {
                    owner_wallet,
                    associated_token_address,
                    mint_account: metadata.mint.to_string(),
                    metadata_account: metadata_pubkey.to_string(),
                };
                nft_holders.push(holder);
            }
        }
    }

    let prefix = if let Some(update_authority) = update_authority {
        update_authority
    } else if let Some(candy_machine_id) = candy_machine_id {
        candy_machine_id
    } else {
        return Err(anyhow!(
            "Must specify either --update-authority or --candy-machine-id"
        ));
    };

    let mut file = File::create(format!("{}/{}_snapshot.json", output, prefix))?;
    serde_json::to_writer(&mut file, &nft_holders)?;

    Ok(())
}

fn get_mints_by_update_authority(
    client: &RpcClient,
    update_authority: &String,
) -> Result<Vec<(Pubkey, Account)>> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 1, // key
            bytes: MemcmpEncodedBytes::Base58(update_authority.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        },
        with_context: None,
    };

    let accounts = client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)?;

    Ok(accounts)
}

pub fn snapshot_cm_accounts(
    client: &RpcClient,
    update_authority: &String,
    output: &String,
) -> Result<()> {
    let accounts = get_cm_accounts_by_update_authority(client, update_authority)?;

    let mut config_accounts = Vec::new();
    let mut candy_machine_accounts = Vec::new();

    for (pubkey, account) in accounts {
        let length = account.data.len();

        // Candy machine accounts have a fixed length, config accounts do not.
        if length == 529 {
            candy_machine_accounts.push(CandyMachineAccount {
                address: pubkey.to_string(),
                data_len: length,
            });
        } else {
            config_accounts.push(ConfigAccount {
                address: pubkey.to_string(),
                data_len: length,
            });
        }
    }
    let candy_machine_program_accounts = CandyMachineProgramAccounts {
        config_accounts,
        candy_machine_accounts,
    };

    let mut file = File::create(format!("{}/{}_accounts.json", output, update_authority))?;
    serde_json::to_writer(&mut file, &candy_machine_program_accounts)?;

    Ok(())
}

fn get_cm_accounts_by_update_authority(
    client: &RpcClient,
    update_authority: &String,
) -> Result<Vec<(Pubkey, Account)>> {
    let candy_machine_program_id = Pubkey::from_str(CANDY_MACHINE_PROGRAM_ID)?;
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 8, // key
            bytes: MemcmpEncodedBytes::Base58(update_authority.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        },
        with_context: None,
    };

    let accounts = client.get_program_accounts_with_config(&candy_machine_program_id, config)?;

    Ok(accounts)
}

pub fn get_cm_creator_accounts(
    client: &RpcClient,
    candy_machine_id: &String,
) -> Result<Vec<(Pubkey, Account)>> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 1 + // key
            32 + // update auth
            32 + // mint
            4 + // name string length
            MAX_NAME_LENGTH + // name
            4 + // uri string length
            MAX_URI_LENGTH + // uri*
            4 + // symbol string length
            MAX_SYMBOL_LENGTH + // symbol
            2 + // seller fee basis points
            1 + // whether or not there is a creators vec
            4, // creators
            bytes: MemcmpEncodedBytes::Base58(candy_machine_id.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        },
        with_context: None,
    };

    let accounts = client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)?;

    Ok(accounts)
}

fn get_holder_token_accounts(
    client: &RpcClient,
    mint_account: String,
) -> Result<Vec<(Pubkey, Account)>> {
    let filter1 = RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: MemcmpEncodedBytes::Base58(mint_account),
        encoding: None,
    });
    let filter2 = RpcFilterType::DataSize(165);
    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        data_slice: None,
        commitment: Some(CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        }),
    };

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter1, filter2]),
        account_config,
        with_context: None,
    };

    let holders = client.get_program_accounts_with_config(&TOKEN_PROGRAM_ID, config)?;

    Ok(holders)
}

fn parse_token_amount(data: &ParsedAccount) -> Result<u64> {
    let amount = data
        .parsed
        .get("info")
        .ok_or(anyhow!("Invalid data account!"))?
        .get("tokenAmount")
        .ok_or(anyhow!("Invalid token amount!"))?
        .get("amount")
        .ok_or(anyhow!("Invalid token amount!"))?
        .as_str()
        .ok_or(anyhow!("Invalid token amount!"))?
        .parse()?;
    Ok(amount)
}

fn parse_owner(data: &ParsedAccount) -> Result<String> {
    let owner = data
        .parsed
        .get("info")
        .ok_or(anyhow!("Invalid owner account!"))?
        .get("owner")
        .ok_or(anyhow!("Invalid owner account!"))?
        .as_str()
        .ok_or(anyhow!("Invalid owner amount!"))?
        .to_string();
    Ok(owner)
}
