# Metaboss

The Metaplex NFT Swiss-army knife tool. Coming soon.

## Install From Source

Install [Rust](https://www.rust-lang.org/tools/install).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the source:

```bash
git clone git@github.com:samuelvanderwaal/metaboss.git
```

Change directory and check out the `develop` branch:

```bash
cd metaboss
git checkout develop
```

Install or build with Rust:

```bash
cargo install --path ./
```

or

```bash
cargo build --release
```

## Example Usage

Update a list of mint accounts with new URIs:

```bash
metaboss update_metadata_uri_all -k ~/.config/solana/devnet.json --mint-accounts <path_to_json_file>
```

The JSON file should be an array of `mint_accounts` and `new_uri`s. Example:

```json
[
    {
        "mint_account": "xvy...",
        "new_uri": "https://arweave.net/abakdkjdlkjdkjd"
    },
    { "mint_account": "Cns...", "new_uri": "https://arweave.net/kdsbdkjdkj" }
]
```
