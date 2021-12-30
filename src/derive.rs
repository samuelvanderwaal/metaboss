use metaplex_token_metadata::id;
use solana_sdk::pubkey::Pubkey;
use std::{convert::AsRef, str::FromStr};

pub fn get_generic_pda(str_seeds: String, program_id: String) {
    let str_seeds = str_seeds
        .split(",")
        .map(|s| s.into())
        .collect::<Vec<String>>();

    let seeds: Vec<Vec<u8>> = str_seeds
        .into_iter()
        .map(|seed| pubkey_or_bytes(seed))
        .collect();

    let seeds: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();

    let program_id =
        Pubkey::from_str(&program_id).expect("Failed to parse pubkey from program_id!");
    println!("{}", derive_generic_pda(seeds, program_id));
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

pub fn get_cmv2_pda(candy_machine_id: String) {
    let pubkey =
        Pubkey::from_str(&candy_machine_id).expect("Failed to parse pubkey from candy_machine_id!");
    println!("{}", derive_cmv2_pda(&pubkey));
}

fn derive_generic_pda(seeds: Vec<&[u8]>, program_id: Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(&seeds, &program_id);
    pda
}

pub fn derive_metadata_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = id();

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

fn derive_edition_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = id();

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
        "edition".as_bytes(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_generic_pda() {
        let metadata_program_pubkey =
            Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();

        let seeds = vec![
            "metadata".as_bytes(),
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
}
