# Metaboss

The Solana Metaplex NFT 'Swiss Army Knife' tool. 
Current supported features:

* Decode NFT mint account metadata
* Get a list of mint accounts for a given candy machine ID or update authority
* Get a snapshot of current NFT holders for a given candy machine ID or update authority
* Set update authority on a single NFT or list of NFTs
* Update all data fields for a single NFT or list of NFTs

Planned features:

* Use Solana config for default RPC
* Get snapshot of holders who initially minted from a candy machine, whether or not they currently hold the token
* Get snapshots based on verified creators


Suggestions and PRs welcome!


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



## Binaries

* Linux
* Windows



## Options

-r, --rpc <rpc> The RPC endpoint to use for commands. Defaults to `https://api.devnet.solana.com`.

#### Usage

```bash
metaboss -r https://api.mainnet-beta.solana.com <SUBCOMMAND>
```

Please don't abuse public APIs or you may get rate-limited. If you have heavy work to do, use a private RPC such as from [QuickNode](https://www.quicknode.com/) or [Triton](https://rpcpool.com/#/).


## Subcommands

### Decode

Decode a single NFT mint account metadata into a JSON file.

#### Usage

```bash
metaboss decode --mint-account <MINT_ACCOUNT> -o <OUPUT_DIRECTORY>
```

The command will write the metadata JSON file to the output directory with the mint account as the name: e.g. `CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp.json`. The output option defaults to the current directory.

### Decode All

Decode a list of NFT mint accounts metadata into a JSON file.

#### Usage

```bash
metaboss decode_all --json-file <JSON_FILE> -o <OUPUT_DIRECTORY>
```

The JSON file should be an array of mint accounts to be decoded:

```json
["xSy...", "Cnb..." ...]
```

The command will write each metadata JSON file to the output directory as a separate file with the mint account as the name: e.g. `CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp.json`. The output option defaults to the current directory.

### Help

Displays list of commands and options for the program.



### Get Mints

Get mint accounts for a candy machine or an update authority. Specify *either* a candy machine id, *or* an update authority, but not both and at least one.

#### Usage

```bash
metaboss get_mints --candy-machine-id <CANDY_MACHINE_ID> -o <OUTPUT_DIRECTORY>
```

or

```bash
metaboss get_mints --update_authority <UPDATE_AUTHORITY> -o <OUTPUT_DIRECTORY>
```

This creates a JSON file named: `<CANDY_MACHINE_ID/UPDATE_AUTHORITY>_mint_accounts.json` in the specified output directory. The JSON file contains an array of mint accounts.



### Set Update Authority

**Warning: This command modifies your NFT and is for advanced users. Ensure you understand what the command does and
how it affects your NFT. Use with caution.**

Set the update authority on a single NFT's metadata account.

#### Usage

```bash
metaboss set_update_authority --keypair <PATH_TO_KEYPAIR> --mint-account <MINT_ACCOUNT> --new-update-authority <NEW_UPDATE_AUTHORITY>
```

Outputs a TxId to the command line so you can check the result.



### Set Update Authority All

**Warning: This command modifies your NFT and is for advanced users. Ensure you understand what the command does and
how it affects your NFT. Use with caution.**

Set the update authority on a list NFT's metadata accounts.

#### Usage

```bash
metaboss set_update_authority_all --keypair <PATH_TO_KEYPAIR> --json-file <PATH_TO_JSON_FILE>
```

The JSON file should be an array of objects with `mint_account` and `new_update_authority`:

```json
[
    {
        "mint_account": "CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp",
        "new_update_authority": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB"
    },
    {
        "mint_account": "9pVUWcAje2HphJqXKwkhsehGnPM7RFt5syqUK99tLMFM",
        "new_update_authority": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB"
    }
]
```

Outputs a TxId to the command line so you can check the result.



### Update NFT

**Warning: This command modifies your NFT and is for advanced users. Ensure you understand what the command does and
how it affects your NFT. Use with caution.**

Update all [Data](https://github.com/metaplex-foundation/metaplex/blob/f1962b5d6f32b6dc3e77cd8fee07cf9e404c38e8/rust/token-metadata/program/src/state.rs#L73) fields on a single NFT's metadata account by reading new values for it *from* a URI JSON file. 

**Warning: If your NFT was minted from a candy machine, this command will currently remove your candy machine as a creator. If you do not wish to do this, do not use this command. The next release will have a more granular update option.**

#### Usage

```bash
metaboss update_nft --keypair <PATH_TO_KEYPAIR> --mint-account <MINT_ACCOUNT> --new-uri <NEW_URI>
```

Outputs a TxId to the command line so you can check the result.



### Update NFT All

**Warning: This command modifies your NFT and is for advanced users. Ensure you understand what the command does and
how it affects your NFT. Use with caution.**

Update all [Data](https://github.com/metaplex-foundation/metaplex/blob/f1962b5d6f32b6dc3e77cd8fee07cf9e404c38e8/rust/token-metadata/program/src/state.rs#L73) fields on a list of NFTs' metadata accounts by reading new values for it *from* provided new URIs.

**Warning: If your NFTs were minted from a candy machine, this command will currently remove your candy machine as a creator. If you do not wish to do this, do not use this command. The next release will have a more granular update option.**

```bash
metaboss update_nft_all --keypair <PATH_TO_KEYPAIR> --json-file <PATH_TO_JSON_FILE>
```

The JSON file should be an array of objects with `mint_account` and `new_uri` fields, where the `new_uri` is an already existing JSON file stored at the URI:

```json
[
    {
        "mint_account": "CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp",
        "new_uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw"
    },
    {
        "mint_account": "9pVUWcAje2HphJqXKwkhsehGnPM7RFt5syqUK99tLMFM",
        "new_uri": "https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls"
    }
]
```



### Snapshot

Get a snapshot of current holders of NFTs specifying an NFT collection by either candy machine ID or update authority.  Specify *either* a candy machine id, or *or* an update authority, but not both and at least one.

#### Usage

```bash
metaboss snapshot --candy-machine-id <CANDY_MACHINE_ID> -o <OUTPUT_DIRECTORY>
```

or

```bash
metaboss snapshot --update_authority <UPDATE_AUTHORITY> -o <OUTPUT_DIRECTORY>
```

Creates a `snapshot.json` file in the output directory consisting of an array of objects containing `owner_wallet`,  `token_address` and `mint_account` fields.

```json
[
    {
        "owner_wallet": "BJD9JeKEnGU9mqqagoovTsDSr1bSZQSy8pHS8hHmjve6",
        "token_address": "BqNz4yh9q7z4N2cAdaqMCk7Y9oLF93Pcwc6hBMBRfLtc"
    },
    {
        "owner_wallet": "BJD9JeKEnGU9mqqagoovTsDSr1bSZQSy8pHS8hHmjve6",
        "token_address": "3rDbzaZJ79pz7HPR4cTABxnpGJpeKJuMf32SPfFHvNUg"
    }
]
```



## Example Usage

Update a list of mint accounts with new URIs:

```bash
metaboss update_nft_all -k ~/.config/solana/devnet.json --json-file new_uri.json
```

The JSON file should be an array of `mint_accounts` and `new_uri`s. Example:

```json
[
    {
        "mint_account": "CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp",
        "new_uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw"
    },
    {
        "mint_account": "9pVUWcAje2HphJqXKwkhsehGnPM7RFt5syqUK99tLMFM",
        "new_uri": "https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls"
    }
]
```

Update a single NFT with a new `update_authority`:

```bash
metaboss set_update_authority -k ~/.config/solana/devnet.json --mint-account CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp --new-update-authority 42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB
```
