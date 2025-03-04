use crate::constants::{MASTER_EDITION_PREFIX, METADATA_PREFIX, USER_PREFIX};
use crate::update::{parse_keypair, parse_solana_config};
use metaboss_lib::derive::{derive_collection_delegate_pda, derive_token_record_pda};
use mpl_token_metadata::ID;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token::ID as token_program_id;
use std::{convert::AsRef, str::FromStr};

pub fn get_token_account_pda(mint: String, owner: Option<String>, token_22: bool) {
    let mint = Pubkey::from_str(&mint).expect("Failed to parse pubkey from mint!");

    let owner = if let Some(owner) = owner {
        Pubkey::from_str(&owner).expect("Failed to parse pubkey from owner!")
    } else {
        let solana_opts = parse_solana_config();
        let config_keypair = parse_keypair(None, solana_opts);
        config_keypair.pubkey()
    };

    let program_id = if token_22 {
        Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
            .expect("Failed to parse pubkey from token 22 (extensions) program!")
    } else {
        token_program_id
    };

    println!("{}", derive_token_account_pda(&mint, &owner, &program_id))
}

pub fn get_generic_pda(str_seeds: String, program_id: String) {
    let seeds: Vec<Vec<u8>> = str_seeds
        .split(',')
        .map(|s| s.trim())
        .flat_map(parse_seed)
        .collect();

    let seeds: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    let program_id =
        Pubkey::from_str(&program_id).expect("Failed to parse pubkey from program_id!");
    println!("{}", derive_generic_pda(seeds, program_id));
}

fn parse_seed(s: &str) -> Vec<Vec<u8>> {
    if s.starts_with('"') && s.ends_with('"') {
        // Handle quoted strings
        vec![s[1..s.len() - 1].as_bytes().to_vec()]
    } else if s.chars().all(|c| c.is_ascii_digit()) {
        // Handle numbers: each digit becomes a separate u8
        s.chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .map(|digit| vec![digit])
            .collect()
    } else {
        // Default to pubkey_or_bytes for other cases
        vec![pubkey_or_bytes(s.to_string())]
    }
}

fn pubkey_or_bytes(seed: String) -> Vec<u8> {
    let res = Pubkey::from_str(&seed);
    let value: Vec<u8> = match res {
        Ok(pubkey) => pubkey.as_ref().to_vec(),
        Err(_) => seed.as_bytes().to_owned(),
    };

    value
}

pub fn get_metadata_pda(mint_account: String) {
    let pubkey =
        Pubkey::from_str(&mint_account).expect("Failed to parse pubkey from mint account!");
    println!("{}", derive_metadata_pda(&pubkey));
}

pub fn get_edition_pda(mint_account: String) {
    let pubkey =
        Pubkey::from_str(&mint_account).expect("Failed to parse pubkey from mint account!");
    println!("{}", derive_edition_pda(&pubkey));
}

pub fn get_edition_marker_pda(mint_account: String, edition_num: u64) {
    let pubkey =
        Pubkey::from_str(&mint_account).expect("Failed to parse pubkey from mint account!");
    println!("{}", derive_edition_marker_pda(&pubkey, edition_num));
}

pub fn get_cmv2_pda(candy_machine_id: String) {
    let pubkey =
        Pubkey::from_str(&candy_machine_id).expect("Failed to parse pubkey from candy_machine_id!");
    println!("{}", derive_cmv2_pda(&pubkey));
}

pub fn get_cmv3_pda(candy_machine_id: String) {
    let pubkey =
        Pubkey::from_str(&candy_machine_id).expect("Failed to parse pubkey from candy_machine_id!");
    println!("{}", derive_cmv3_pda(&pubkey));
}

pub fn get_token_record_pda(mint_account: String, token_account: String) {
    let mint_pubkey =
        Pubkey::from_str(&mint_account).expect("Failed to parse pubkey from mint account!");
    let token_pubkey =
        Pubkey::from_str(&token_account).expect("Failed to parse pubkey from token account!");

    println!("{}", derive_token_record_pda(&mint_pubkey, &token_pubkey));
}

pub fn get_collection_delegate(mint: Pubkey, authority: Pubkey, delegate: Pubkey) {
    println!(
        "{:?}",
        derive_collection_delegate_pda(&mint, &delegate, &authority)
    );
}

fn derive_generic_pda(seeds: Vec<&[u8]>, program_id: Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(&seeds, &program_id);
    pda
}

pub fn derive_token_account_pda(mint: &Pubkey, owner: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_associated_token_address_with_program_id(owner, mint, program_id)
}

pub fn derive_metadata_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = ID;

    let seeds = &[
        METADATA_PREFIX.as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

pub fn derive_edition_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = ID;

    let seeds = &[
        METADATA_PREFIX.as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
        MASTER_EDITION_PREFIX.as_bytes(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

pub fn derive_edition_marker_pda(pubkey: &Pubkey, edition_num: u64) -> Pubkey {
    let metaplex_pubkey = ID;

    let num: String = (edition_num / 248).to_string();

    let seeds = &[
        METADATA_PREFIX.as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
        MASTER_EDITION_PREFIX.as_bytes(),
        num.as_bytes(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

pub fn derive_cmv2_pda(pubkey: &Pubkey) -> Pubkey {
    let cmv2_pubkey = Pubkey::from_str("cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ")
        .expect("Failed to parse pubkey from candy machine program id!");

    let seeds = &["candy_machine".as_bytes(), pubkey.as_ref()];

    let (pda, _) = Pubkey::find_program_address(seeds, &cmv2_pubkey);
    pda
}

pub fn derive_cmv3_pda(pubkey: &Pubkey) -> Pubkey {
    let cmv3_pubkey = Pubkey::from_str("CndyV3LdqHUfDLmE5naZjVN8rBZz4tqhdefbAnjHG3JR")
        .expect("Failed to parse pubkey from candy machine program id!");

    let seeds = &["candy_machine".as_bytes(), pubkey.as_ref()];

    let (pda, _) = Pubkey::find_program_address(seeds, &cmv3_pubkey);
    pda
}

pub fn derive_collection_authority_record(
    mint: &Pubkey,
    collection_authority: &Pubkey,
) -> (Pubkey, u8) {
    let metaplex_pubkey = ID;

    let seeds = &[
        METADATA_PREFIX.as_bytes(),
        metaplex_pubkey.as_ref(),
        mint.as_ref(),
        "collection_authority".as_bytes(),
        collection_authority.as_ref(),
    ];
    Pubkey::find_program_address(seeds, &metaplex_pubkey)
}

pub fn derive_use_authority_record(mint: &Pubkey, use_authority: &Pubkey) -> (Pubkey, u8) {
    let metaplex_pubkey = ID;

    let use_authority_seeds = &[
        METADATA_PREFIX.as_bytes(),
        metaplex_pubkey.as_ref(),
        mint.as_ref(),
        USER_PREFIX.as_bytes(),
        use_authority.as_ref(),
    ];
    Pubkey::find_program_address(use_authority_seeds, &metaplex_pubkey)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_generic_pda() {
        let metadata_program_pubkey =
            Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();

        let seeds = vec![
            METADATA_PREFIX.as_bytes(),
            metadata_program_pubkey.as_ref(),
            mint_pubkey.as_ref(),
        ];

        let expected_pda =
            Pubkey::from_str("99pKPWsqi7bZaXKMvmwkxWV4nJjb5BS5SgKSNhW26ZNq").unwrap();
        let program_pubkey =
            Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();

        assert_eq!(derive_generic_pda(seeds, program_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_metadata_pda() {
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let expected_pda =
            Pubkey::from_str("99pKPWsqi7bZaXKMvmwkxWV4nJjb5BS5SgKSNhW26ZNq").unwrap();
        assert_eq!(derive_metadata_pda(&mint_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_edition_pda() {
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let expected_pda =
            Pubkey::from_str("2vNgLPdTtfZYMNBR14vL5WXp6jYAvumfHauEHNc1BQim").unwrap();
        assert_eq!(derive_edition_pda(&mint_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_cmv2_pda() {
        let candy_machine_pubkey =
            Pubkey::from_str("3qt9aBBmTSMxyzFEcwzZnFeV4tCZzPkTYVqPP7Bw5zUh").unwrap();
        let expected_pda =
            Pubkey::from_str("8J9W44AfgWFMSwE4iYyZMNCWV9mKqovS5YHiVoKuuA2b").unwrap();
        assert_eq!(derive_cmv2_pda(&candy_machine_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_cmv3_pda() {
        let candy_machine_pubkey =
            Pubkey::from_str("2YkBXpx61ziscLL4aAdXYnjCsoHK5TF9rF9AgkhhLJAX").unwrap();
        let expected_pda =
            Pubkey::from_str("Ei45DbGouAM7NkN5Rk22ep415vbGrbVPEDG9tRfoHb5B").unwrap();
        assert_eq!(derive_cmv3_pda(&candy_machine_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_token_account_pda() {
        let owner_pubkey =
            Pubkey::from_str("8LSSjDHrfzcf3GnyE41F6SxMifpRCBA7NKSnfAEYzU7q").unwrap();

        let token_keg_mint_pubkey =
            Pubkey::from_str("2D49Wa5zMu16p73KBoSyJ596Uf1ShboTGNF51abKU2VE").unwrap();
        let token_22_mint_pubkey =
            Pubkey::from_str("CCFjUhgpDy2am8mqVM9ib8rszyU1bgpyDnLAJ7KjLAVy").unwrap();

        let token_keg_program_id =
            Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
        let token_22_program_id =
            Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap();

        let expected_token_keg_account_pda =
            Pubkey::from_str("z2pwPKE1vKSUq9gEjejj2Y9FFq3Unh2eF94LYFQPMES").unwrap();
        let expected_token_22_account_pda =
            Pubkey::from_str("87dZdrtQwbbBTSYbVN6DfQUhugx7YZmrha3ZNKhKarKa").unwrap();

        assert_eq!(
            derive_token_account_pda(&token_keg_mint_pubkey, &owner_pubkey, &token_keg_program_id),
            expected_token_keg_account_pda
        );

        assert_eq!(
            derive_token_account_pda(&token_22_mint_pubkey, &owner_pubkey, &token_22_program_id),
            expected_token_22_account_pda
        );
    }
}
