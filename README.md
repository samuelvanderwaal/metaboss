# Metaboss

The Solana Metaplex NFT 'Swiss Army Knife' tool.

Current supported features:

-   Decode the metadata of a token mint account

-   Mint new NFTs from a JSON file

-   Set `primary_sale_happened` bool on an NFT's metadata

-   Set `update_authorty` address on an NFT's metadata

-   Verify a creator by signing the metadata accounts for all tokens in a list, for a given candy machine id or a single mint account

-   Get a snapshot of current NFT holders for a given candy machine ID or update authority

-   Get a list of mint accounts for a given candy machine ID or update authority

-   Get a list of all candy machine state and config accounts for a given update authority

-   Update all metadata Data struct fields for a NFT

-   Update just the URI for a NFT

Suggestions and PRs welcome!

**Note: This is experimental software for a young ecosystem. Use at your own risk. The author is not responsible for misuse of the software or failing to test specific commands before using on production NFTs.**

## Contact

Email: sam@vanderwaal.dev

Twitter: [@samvwaal](https://twitter.com/samvwaal)

Discord: @archaeopteryx#7615

## Binaries

Linux, MacOS and Windows binaries available in [releases](https://github.com/samuelvanderwaal/metaboss/releases), thanks to CI work done by [Kartik Soneji](https://github.com/KartikSoneji).

## Install From Source

Install [Rust](https://www.rust-lang.org/tools/install).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the source:

```bash
git clone git@github.com:samuelvanderwaal/metaboss.git
```

or

```bash
git clone https://github.com/samuelvanderwaal/metaboss.git
```

Change directory and check out the `main` branch:

```bash
cd metaboss
git checkout main
```

Install or build with Rust:

```bash
cargo install --path ./
```

or

```bash
cargo build --release
```

## Examples

### Update the URI of an existing NFT

```bash
metaboss update uri -k ~/.config/solana/devnet.json -a CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp -u https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls
```

### Mint a new NFT

Prepare a JSON file.

```json
{
    "name": "TestNFT2",
    "symbol": "TNFT",
    "uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw",
    "seller_fee_basis_points": 200,
    "creators": [
        {
            "address": "AVdBTNhDqYgXGaaVkqiaUJ1Yqa61hMiFFaVRtqwzs5GZ",
            "verified": false,
            "share": 50
        },
        {
            "address": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB",
            "verified": false,
            "share": 50
        }
    ]
}
```

Call command. In this case we do not set `--receiver` so we mint directly to the `keypair` address.

```bash
metaboss mint one -k ~/.config/solana/devnet.json -d ./new_nft.json
```

### Snapshot Candy Machine Mint Accounts

We call the command with no output specified so it creates the file in the current directory.

```bash
metaboss snapshot mints -c BHZWQEtGRMs7voC7vDyVQCXawB1P6UvxG899ATGwxmaR
```

The file `BHZWQEtGRMs7voC7vDyVQCXawB1P6UvxG899ATGwxmaR_mint_accounts.json` is created with the contents:

```json
[
    "D5ycm2mgBWDR37QVkvM389x84V4ux48bSeHLeiHPtX28",
    "4kYdMRRYtXjmkusgKEBntSXLDhqkHNE57GF3RPdtx6MW",
    "J8xuCFCeBRESoXewtMwrrpVUGikUG3B1WznNdLffyymz",
    "4gRtRjrbD7g5ZKUvSVA1tYMK9LZqz6uWuSc3rKeinySh"
]
```

## Options

-r, --rpc <rpc> The RPC endpoint to use for commands.

Metaboss will try to read your Solana config settings for both the RPC endpoint and also the Commitment setting by reading from `$HOME/.config/solana/cli/config.yml`. If it can't find a config file it defaults to using ` https://api.devnet.solana.com` and `confirmed`.

Running Metaboss with the `--rpc` option will override the above with whatever RPC endpoint the user provides.

##### Usage

```bash
metaboss -r https://api.mainnet-beta.solana.com <SUBCOMMAND>
```

Please don't abuse public APIs or you may get rate-limited. If you have heavy work to do, use a private RPC such as from [QuickNode](https://www.quicknode.com/) or [Triton](https://rpcpool.com/#/).

## Subcommands

### Decode

The Decode subcommand retrieves binary data from accounts on chain and decodes it into human-readable JSON files.

#### Decode Mint

Decodes a mint account's metadata into a JSON file. It accepts either a single account or a list of accounts.

##### Usage

```bash
metaboss decode mint --account <MINT_ACCOUNT> -o <OUPUT_DIRECTORY>
```

The command will write the metadata JSON file to the output directory with the mint account as the name: e.g. `CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp.json`. The output option defaults to the current directory.

```bash
metaboss decode mint --list-file <LIST_FILE> -o <OUPUT_DIRECTORY>
```

The JSON list file should be an array of mint accounts to be decoded:

```json
["xSy...", "Cnb..." ...]
```

The command will write each metadata JSON file to the output directory as a separate file with the mint account as the name: e.g. `CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp.json`. The output option defaults to the current directory.

### Help

Displays list of commands and options for the program.

### Mint

Mint new NFTs from JSON files.

#### Mint One

Mint a single NFT from a JSON file.

##### Usage

```bash
metaboss mint one --keypair <KEYPAIR> --nft-data-file <PATH_TO_NFT_DATA_FILE> --receiver <RECEIVER_ADDRESS>
```

The JSON files should contain all the necessary data required to create an NFT's metadata fields. Creator `verified` fields must be false unless the creator is also the `keypair`.

Example JSON file:

```json
{
    "name": "TestNFT1",
    "symbol": "TNFT",
    "uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw",
    "seller_fee_basis_points": 100,
    "creators": [
        {
            "address": "PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8",
            "verified": false,
            "share": 100
        }
    ]
}
```

If `receiver` is set, the NFT will be minted directly to the receiver's address, otherwise it is minted to `keypair`. Observant users may note that with a simple bash script this allows airdrops to be deployed with Metaboss.

#### Mint List

Mint multiple NFTs from a list of JSON files.

##### Usage

```bash
metaboss mint list --keypair <KEYPAIR> --nft-data-dir <PATH_TO_NFT_DATA_FILE> --receiver <RECEIVER_ADDRESS>
```

This command functions the same as `mint one` except instead of a single JSON file, provide a path to a directory with multiple JSON files, one for each NFT to be minted.

By default, new NFTs are minted as mutable, to make them immutable use the `--immutable` option.

### Set

**Warning: These commands modify your NFT and are for advanced users. Use with caution.**

Set non-Data struct values for a NFT.

#### Set Primary-Sale-Happened

Set `primary_sale_happened` to be `true`.

```bash
metaboss set primary-sale-happened --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT>
```

Outputs a TxId to the command line so you can check the result.

#### Set Update-Authority

Set `update_authority` to a different public key.

```bash
metaboss set update-authority --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-update-authority <NEW_UPDATE_AUTHORITY>
```

#### Set Update-Authority-All

Set `update_authority` to a different public key for a list of NFTs.

```bash
metaboss set update-authority-all --keypair <PATH_TO_KEYPAIR> --mint-accounts-file <PATH_TO_MINT_ACCOUNTS> --new-update-authority <NEW_UPDATE_AUTHORITY>
```

The mint accounts file should be a JSON file with an array of NFT mint accounts to be updated:

```json
[
    "C2eGm8iQPnKVWxakyo8QhwJUvYrZHKF52DPQuAejpTWG",
    "8GcRqxy4VAocTcAkoxCXkPCEmM36HMtjBc8ZarWhAD6o",
    "CK2npuck3WTRNFXSdZv8YjudJJEa69EVGd6GFfeSzfGP"
]
```



### Sign

**Warning: These commands modify your NFT and are for advanced users. Use with caution.**

Sign metadata for an unverified creator.

#### Sign One

Sign the metadata for a single mint account.

##### Usage

```bash
metaboss sign one --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT>
```

Outputs a TxId to the command line so you can check the result.

#### Sign All

Sign all metadata from a JSON list or for a given candy machine id.

##### Usage

```bash
metaboss sign all --keypair <PATH_TO_KEYPAIR> --candy-machine-id <CANDY_MACHINE_ID>
```

```bash
metaboss sign all --keypair <PATH_TO_KEYPAIR> --mint-accounts-file <PATH_TO_MINT_ACCOUNTS_FILE>
```

For the latter usage, the mint accounts file should be a JSON file with a list of mint accounts to be signed:

```json
[
    "C2eGm8iQPnKVWxakyo8QhwJUvYrZHKF52DPQuAejpTWG",
    "8GcRqxy4VAocTcAkoxCXkPCEmM36HMtjBc8ZarWhAD6o",
    "CK2npuck3WTRNFXSdZv8YjudJJEa69EVGd6GFfeSzfGP"
]
```

Outputs a TxId to the command line so you can check the result.

### Snapshot

Get snapshots of various blockchain states.

#### Snapshot CM-Accounts

Snapshot all candy machine config and state accounts for a given update_authority.

##### Usage

```bash
metaboss snapshot cm-accounts --update-authority <UPDATE_AUTHORITY> --output <OUTPUT_DIR>
```

Creates a JSON file in the output directory with the name format of `<UPDATE_AUTHORITY>_accounts.json`, consisting of an object with the fields `config_accounts` and `candy_machine_accounts`:

```json
{
    "config_accounts": [
        {
            "address": "2XBqwwTLf24ACPR3BDSEKCB95PZiAwYySeX1LyN3FKDL",
            "data_len": 1456
        },
        {
            "address": "9tNkktGZhLiWHkc4JhoTYvMLXEVA8qauSVeFwyiRPCsT",
            "data_len": 1216
        }
    ],
    "candy_machine_accounts": [
        {
            "address": "DwoPaGFxJpGRq3kZQBNfBroCGaS9Hdg2rpFHJpD2iBhW",
            "data_len": 529
        },
        {
            "address": "CpFAvcReAkmxWiL7jwDjBKD9jX1Bi1Lky4bHwMkgCuxc",
            "data_len": 529
        }
    ]
}
```

#### Snapshot Holders

Snapshot all current holders of NFTs filtered by candy_machine_id or update_authority

##### Usage

```bash
metaboss snapshot holders --candy-machine-id <CANDY_MACHINE_ID> --output <OUTPUT_DIR>
```

or

```bash
metaboss snapshot holders --update-authority <UPDATE_AUTHORITY> --output <OUTPUT_DIR>
```

Creates a JSON file in the output directory with the name format of `<CANDY_MACHINE_ID/UPDATE_AUTHORITY>_holders.json` consisting of an array of objects with the following fields:

-   owner wallet -- the holder of the token
-   associated token account -- the token account the NFT is stored at
-   mint account -- the token mint account for the NFT
-   metadata account -- the metadata account decorating the mint account that defines the NFT

Example file:

```json
[
    {
        "owner_wallet": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB",
        "associated_token_address": "7yGA66LYDU7uoPW2x9jrUKaDWTs9jqZ5cSNKR1VaLQdw",
        "mint_account": "C2eGm8iQPnKVWxakyo8QhwJUvYrZHKF52DPQuAejpTWG",
        "metadata_account": "8WTA3sLxwRNDKHxZFbn2CFo3FX1ZP59EqrvuDPLbmmWV"
    }
]
```

#### Snapshot Mints

Snapshot all mint accounts for a given candy machine id or update authority

##### Usage

```bash
metaboss snapshot mints --candy-machine-id <CANDY_MACHINE_ID> --output <OUTPUT_DIR>
```

or

```bash
metaboss snapshot mints --update-authority <UPDATE_AUTHORITY> --output <OUTPUT_DIR>
```

Creates a JSON file in the output directory with the name format of `<CANDY_MACHINE_ID/UPDATE_AUTHORITY>_mint_accounts.json` consisting of an array of mint accounts.

```json
[
    "CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp",
    "5pgGJ5npeMxBzTiQctDgoofEVGSwZMYm3QMz4F4NDShz",
    "8GcRqxy4VAocTcAkoxCXkPCEmM36HMtjBc8ZarWhAD6o"
]
```

### Update

**Warning: These commands modify your NFT and are for advanced users. Use with caution.**

Update various aspects of an NFT.

#### Update Data

Update the `Data` struct on a NFT from a JSON file.

##### Usage

```bash
metaboss update data --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-data-file <PATH_TO_NEW_DATA_FILE>
```

The JSON file should include all the fields of the metadata `Data` struct and should match `creator` `verified` bools for existing creators. E.g. if your NFT was minted by the Metaplex Candy Machine program, and you wish to keep your candy machine as a verified creator _you must add the candy machine to your creators array with `verified` set to `true`_.

**Make sure you understand how the Metaplex Metadata `Data` struct works and how this command will affect your NFT. Always test on `devnet` before running on mainnet. **

```json
{
    "name": "FerrisCrab #4",
    "symbol": "FERRIS",
    "uri": "https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls",
    "seller_fee_basis_points": 100,
    "creators": [
        {
            "address": "<YOUR_CANDY_MACHINE_ID>",
            "verified": true,
            "share": 0
        },
        {
            "address": "<KEYPAIR_CREATOR>",
            "verified": true,
            "share": 50
        },
        {
            "address": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB",
            "verified": false,
            "share": 50
        }
    ]
}
```

Outputs a TxId to the command line so you can check the result.

#### Update URI

Update the metadata URI, keeping the rest of the `Data` struct the same.

##### Usage

```bash
metaboss update uri --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-uri <NEW_URI>
```
