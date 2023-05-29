use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;

use regex::Regex;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;

#[test]
#[ignore]
fn mint_first_edition() {
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    println!("Minting first edition...");

    // Arrange

    // Mint master edition
    let output = Command::new("metaboss")
        .args(["mint", "one", "-d", "tests/files/new_nft.json", "-e", "10"])
        .output()
        .expect("failed to execute process");

    println!("mint status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();

    let re = Regex::new(r"Mint account: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let master_edition = &cap[1];

    // Act

    // Mint first edition
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "1"]);
    println!("command: {:?}", command);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r"Edition with mint: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let edition_pubkey = Pubkey::from_str(&cap[1]).unwrap();

    let edition = metaboss_lib::decode::decode_edition_from_mint(&client, edition_pubkey).unwrap();

    // Assert
    assert_eq!(edition.edition, 1);
}

#[test]
#[ignore]
fn mint_next_edition_marker_1() {
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    println!("Minting first edition...");

    // Arrange

    // Mint master edition
    let output = Command::new("metaboss")
        .args(["mint", "one", "-d", "tests/files/new_nft.json", "-e", "10"])
        .output()
        .expect("failed to execute process");

    println!("mint status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();

    let re = Regex::new(r"Mint account: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let master_edition = &cap[1];

    // Act

    // Mint first edition
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-s", "1"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r"Edition with mint: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let edition_pubkey = Pubkey::from_str(&cap[1]).unwrap();

    let edition = metaboss_lib::decode::decode_edition_from_mint(&client, edition_pubkey).unwrap();

    assert_eq!(edition.edition, 1);

    // Mint fifth edition to leave a gap.
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-s", "5"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r"Edition with mint: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let edition_pubkey = Pubkey::from_str(&cap[1]).unwrap();

    let edition = metaboss_lib::decode::decode_edition_from_mint(&client, edition_pubkey).unwrap();

    assert_eq!(edition.edition, 5);

    // Assert

    // Mint next edition, which should mint the second edition.
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "1"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r"Edition with mint: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let edition_pubkey = Pubkey::from_str(&cap[1]).unwrap();

    let edition = metaboss_lib::decode::decode_edition_from_mint(&client, edition_pubkey).unwrap();

    print!("edition: {:?}", edition);

    assert_eq!(edition.edition, 2);
}

#[test]
#[ignore = "Takes a long time to mint out all the editions, so run manually"]
fn mint_next_edition_marker_2() {
    let client = RpcClient::new_with_commitment(
        "http://localhost:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    println!("Minting first edition...");

    // Arrange

    // Mint master edition
    let output = Command::new("metaboss")
        .args([
            "mint",
            "one",
            "-d",
            "tests/files/new_nft.json",
            "-e",
            "1000",
        ])
        .output()
        .expect("failed to execute process");

    println!("mint status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();

    let re = Regex::new(r"Mint account: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let master_edition = &cap[1];

    // Act

    // Mint out first marker
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "50"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "50"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "50"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "50"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "48"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    // Mint edition in second marker with a gap.
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-s", "250"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r"Edition with mint: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let edition_pubkey = Pubkey::from_str(&cap[1]).unwrap();

    let edition = metaboss_lib::decode::decode_edition_from_mint(&client, edition_pubkey).unwrap();

    assert_eq!(edition.edition, 250);

    // Assert

    // Mint next edition, which should mint the second edition.
    let mut command = Command::new("metaboss");
    command.args(["mint", "editions", "-a", master_edition, "-n", "1"]);

    let output = command.output().expect("failed to execute process");

    io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let output_string = String::from_utf8(output.stdout).unwrap();
    let re = Regex::new(r"Edition with mint: (\S+)").unwrap();
    let cap = re.captures(&output_string).unwrap();

    let edition_pubkey = Pubkey::from_str(&cap[1]).unwrap();

    let edition = metaboss_lib::decode::decode_edition_from_mint(&client, edition_pubkey).unwrap();

    assert_eq!(edition.edition, 249);
}
