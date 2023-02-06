## Update

**Warning: These commands modify your NFT and are for advanced users. Use with caution.**

Update various aspects of an NFT.

See also **Set** commands for updatable values that cannot be reversed (e.g. set immutable).

### Update Data

Update the `Data` struct on a NFT from a JSON file.

#### Usage

```bash
metaboss update data --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-data-file <PATH_TO_NEW_DATA_FILE>
```

The JSON file should include all the fields of the metadata `Data` struct and should match `creator` `verified` bools for existing creators. E.g. if your NFT was minted by the Metaplex Candy Machine program, and you wish to keep your candy machine as a verified creator _you must add the candy machine to your creators array with `verified` set to `true`_.

Note: The on-chain `Data` struct is *different* than the external metadata stored at the link in the `uri` field so make you understand the difference before running this command.

**Make sure you understand how the Metaplex Metadata `Data` struct works and how this command will affect your NFT. Always test on `devnet` before running on mainnet.**

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

### Update Data All

Update the `Data` struct on a list of NFTs from JSON files.

#### Usage

```bash
metaboss update data-all --keypair <PATH_TO_KEYPAIR> --data-dir <PATH_TO_DATA_DIR>
```

Each JSON file in the data directory should include the mint account and all the fields of the metadata `Data` struct and should match `creator` `verified` bools for existing creators. E.g. if your NFT was minted by the Metaplex Candy Machine program, and you wish to keep your candy machine as a verified creator _you must add the candy machine to your creators array with `verified` set to `true`_.

Note: The on-chain `Data` struct is *different* than the external metadata stored at the link in the `uri` field so make you understand the difference before running this command.

**Make sure you understand how the Metaplex Metadata `Data` struct works and how this command will affect your NFT. Always test on `devnet` before running on mainnet.**

```json
{
    "mint_account": "CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp",
    "nft_data":
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
    ]}
}
```

Outputs a TxId to the command line so you can check the result.

### Update Name

Update the on-chain name of a NFT, keeping the rest of the `Data` struct the same.

#### Usage

```bash
 metaboss update name --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-name <NEW_NAME>
 ```


### Update Symbol

Update the on-chain symbol of a NFT, keeping the rest of the `Data` struct the same.

#### Usage

```bash
 metaboss update symbol --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-symbol <NEW_SYMBOL>
 ```

 ### Update Symbol All

Update the on-chain symbol of a list of NFTs, keeping the rest of the `Data` struct the same.

#### Usage

```bash
 metaboss update symbol-all --keypair <PATH_TO_KEYPAIR> -L <PATH_TO_LIST_MINT_ADDRESSES> --new-symbol <NEW_SYMBOL>
 ```

### Update Creators

Update the creators of a NFT, keeping the rest of the `Data` struct the same.

#### Usage

```bash
metaboss update creators -k <PATH_TO_KEYPAIR> -a <MINT_ACCOUNT> -c <CREATOR1:SHARE:VERIFIED,CREATOR2:SHARE:VERIFIED>
```

Creators should be a comma-delimited list of creator:share:verified. E.g. 

Example:

```bash
metaboss update creators -k ~/.config/solana/devnet.json -a 4rxTT8pKeYFrFgNBgTspBWVEnMnsAZGwChkjRUtP4Xpi -c 42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB:70:false,AVdBTNhDqYgXGaaVkqiaUJ1Yqa61hMiFFaVRtqwzs5GZ:30:false
```

Using the `--append` flag will set the shares to 0 and append to the existing creators list, otherwise the creators are overwritten with the list you pass in.

### Update Creators All

Same as update creators but takes a mint list instead of a single account.

```bash
metaboss update creators-all  -k ~/.config/solana/devnet.json -L mints.json -c 42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB:70:false,AVdBTNhDqYgXGaaVkqiaUJ1Yqa61hMiFFaVRtqwzs5GZ:30:false
```

### Update URI

Update the metadata URI, keeping the rest of the `Data` struct the same.

#### Usage

```bash
metaboss update uri --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-uri <NEW_URI>
```

### Update URI All

Update the metadata URI for a list of mint accounts, keeping the rest of the `Data` struct the same.

#### Usage

```bash
metaboss update uri-all --keypair <PATH_TO_KEYPAIR> --json-file <PATH_TO_JSON_FILE>
```

```json
[
    {
        "mint_account": "xZ43...",
        "new_uri": "https://arweave.net/N36gZYJ6PEH8OE11i0MppIbPG4VXKV4iuQw1zaq3rls"
    },
        {
        "mint_account": "71bk...",
        "new_uri": "https://arweave.net/FPGAv1XnyZidnqquOdEbSY6_ES735ckcDTdaAtI7GFw"
    }
]
```

### Update Seller Fee Basis Points

Update the seller fee basis points field on an NFT, keeping the rest of the `Data` struct the same.

#### Usage

```bash
metaboss update sfbp --keypair <PATH_TO_KEYPAIR> -a <MINT_ACCOUNT> -n <NEW_SELLER_FEE_BASIS_POINTS_VALUE>
```

### Update Seller Fee Basis Points All

Update the seller fee basis points field on a a list of NFTs, keeping the rest of the `Data` struct the same.

#### Usage

```bash
metaboss update sfbp-all --keypair <PATH_TO_KEYPAIR> -L <PATH_TO_MINT_LIST.json> -n <NEW_SELLER_FEE_BASIS_POINTS_VALUE>
```

### Update Rule Set

Update a Metaplex pNFT's rule set pubkey.

```
USAGE:
    metaboss update rule-set [OPTIONS] --mint <mint> --new-rule-set <new-rule-set>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --keypair <keypair>              Path to the creator's keypair file
    -l, --log-level <log-level>          Log level [default: off]
    -a, --mint <mint>                    Mint account of token to transfer
    -n, --new-rule-set <new-rule-set>    New rule set pubkey
    -r, --rpc <rpc>                      RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>              Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss update rule-set --mint <MINT_ADDRESS> --new-rule-set <NEW_RULE_SET_PUBKEY>
```

E.g.:

```bash
metaboss update rule-set --mint 2KGQLgypChErw3kKPqG26uyUjVtZj8QSJg2AUNR7BWdM -n D4YHFZPWASGpvBDJSUrPtqZqxTgTm7eL5rikBY9Y5dwf
```

### Update Rule Set All

Update the rule set of a batch of pNFTs.

USAGE:
    metaboss update rule-set-all [OPTIONS] --new-rule-set <new-rule-set>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --batch-size <batch-size>        Maximum number of concurrent requests [default: 10]
    -c, --cache-file <cache-file>        Cache file
    -k, --keypair <keypair>              Path to the creator's keypair file
    -l, --log-level <log-level>          Log level [default: off]
    -L, --mint-list <mint-list>          Path to the mint list file
    -n, --new-rule-set <new-rule-set>    New rule set pubkey
        --retries <retries>              Maximum retries: retry failed items up to this many times [default: 1]
    -r, --rpc <rpc>                      RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>              Timeout to override default value of 90 seconds [default: 90]

#### Usage

```bash
metaboss update rule-set-all -L <MINT_LIST> -n <NEW_RULE_SET_ADDRESS>
```

E.g.:

```bash
metaboss update rule-set-all -L rule_set_mints.json -n 1CfDY5sYBnspaXvjnN3y9WRdaoD5v3HXrZDrWhjZZTN
```


### Update Clear Rule Set

Remove the rule set on a pNFT.

```
USAGE:
    metaboss update clear-rule-set [OPTIONS] --mint <mint>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --keypair <keypair>        Path to the creator's keypair file
    -l, --log-level <log-level>    Log level [default: off]
    -a, --mint <mint>              Mint account of token to transfer
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss update clear-rule-set --mint <MINT_ADDRESS>
```

E.g.:

```bash
metaboss update clear-rule-set --mint 2KGQLgypChErw3kKPqG26uyUjVtZj8QSJg2AUNR7BWdM
```

### Update Clear Rule Set All

Remove the rule set of a batch of pNFTs

USAGE:
    metaboss update clear-rule-set-all [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --batch-size <batch-size>    Maximum number of concurrent requests [default: 10]
    -c, --cache-file <cache-file>    Cache file
    -k, --keypair <keypair>          Path to the creator's keypair file
    -l, --log-level <log-level>      Log level [default: off]
    -L, --mint-list <mint-list>      Path to the mint list file
        --retries <retries>          Maximum retries: retry failed items up to this many times [default: 1]
    -r, --rpc <rpc>                  RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>          Timeout to override default value of 90 seconds [default: 90]

#### Usage

```bash
metaboss update clear-rule-set-all -L <MINT_LIST>
```

E.g.:

```bash
metaboss update rule-set-all -L rule_set_mints.json