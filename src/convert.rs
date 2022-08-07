use anyhow::Result;
use bs58;
use hex;

pub fn hex_to_base58(hex: &str) -> Result<()> {
    let bytes = hex::decode(hex)?;
    let base58 = bs58::encode(&bytes).into_string();
    println!("{base58}");
    Ok(())
}

pub fn base58_to_hex(base58: &str) -> Result<()> {
    let bytes = bs58::decode(base58).into_vec()?;
    let hex = hex::encode(&bytes);
    println!("{hex}");
    Ok(())
}

pub fn bytes_to_hex(bytes_str: &str) {
    let bytes: Vec<u8> = bytes_str
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|c| {
            c.parse()
                .unwrap_or_else(|_| panic!("failed to parse {}", c))
        })
        .collect();
    println!("{:?}", hex::encode(bytes));
}

pub fn hex_to_bytes(hex: &str) -> Result<()> {
    let bytes = hex::decode(hex)?;
    println!("{:?}", bytes);
    Ok(())
}

pub fn base58_to_bytes(base58: &str) -> Result<()> {
    let bytes = bs58::decode(base58).into_vec()?;
    println!("{:?}", bytes);
    Ok(())
}
