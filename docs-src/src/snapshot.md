## Snapshot

Get snapshots of various blockchain states.

### Snapshot CM-Accounts

Snapshot all candy machine config and state accounts for a given update_authority.

#### Usage

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

### Snapshot Holders

Snapshot all current holders of NFTs filtered by verified candy_machine_id/first creator or update_authority.
**Note:** Update authority can be faked so use that option with caution.

#### Usage

```bash
metaboss snapshot holders --creator <CREATOR_ADDRESS> -p <POSITION> --output <OUTPUT_DIR>
```

Use the position to indicate which creator in the creators array to filter by; defaults to the first one (position 0).

or

```bash
metaboss snapshot holders --update-authority <UPDATE_AUTHORITY> --output <OUTPUT_DIR>
```

**For candy machine v2, you can add the `--v2` option when using it with candy machine id.**
Candy machine v2 has a separate creator id from the candy machine account id. 

```bash
metaboss snapshot holders --creator <CANDY_MACHINE_ID> --v2 --output <OUTPUT_DIR>
```
where <CANDY_MACHINE_ID> is the candy machine id retrieved from the cache file.

Creates a JSON file in the output directory with the name format of `<CREATOR/UPDATE_AUTHORITY>_holders.json` consisting of an array of objects with the following fields:

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

### Snapshot Mints

Snapshot all mint accounts for a given verified candy machine id/first creator or update authority

#### Usage

```bash
metaboss snapshot mints --creator <FIRST_CREATOR> --output <OUTPUT_DIR>
```

Use the position to indicate which creator in the creators array to filter by; defaults to the first one (position 0).

or

```bash
metaboss snapshot mints --update-authority <UPDATE_AUTHORITY> --output <OUTPUT_DIR>
```

**For candy machine v2, you can add the `--v2` option when using it with candy machine id.**
Candy machine v2 has a separate creator id from the candy machine account id.

```bash
metaboss snapshot mints --creator <CANDY_MACHINE_ID> --v2 --output <OUTPUT_DIR>
```
where <CANDY_MACHINE_ID> is the candy machine id retrieved from the cache file.

Creates a JSON file in the output directory with the name format of `<CANDY_MACHINE_ID/UPDATE_AUTHORITY>_mint_accounts.json` consisting of an array of mint accounts.

```json
[
    "CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp",
    "5pgGJ5npeMxBzTiQctDgoofEVGSwZMYm3QMz4F4NDShz",
    "8GcRqxy4VAocTcAkoxCXkPCEmM36HMtjBc8ZarWhAD6o"
]
```

### Indexed Data

Metaboss now has experimental support for running snapshot commands using off-chain, indexed data from https://theindex.io/. Other indexers or methods may be supported later. To use this feature, you need to sign up for a free account with TheIndex to get an API key.

### Snapshot Indexed Mints

#### Usage

```bash
metaboss snapshot indexed-mints --creator <FIRST_VERIFIED_CREATOR> --api-key <THEINDEX.IO_API_KEY>
```

### Snapshot Indexed Holders

#### Usage

```bash
metaboss snapshot indexed-holders --creator <FIRST_VERIFIED_CREATOR> --api-key <THEINDEX.IO_API_KEY>
```

### Snapshot Prints

Snapshot the print editions of a given master edition. This returns a JSON object of edition mints where the key is the edition number and the value is the mint address.

E.g.:
```json
{
  "1": "CgCmZJCBeJs9m596NzqxLg3HB8eerHHWXQPiiigB3fpt",
  "2": "C7kfCVwadqrQwjCBewE147r3xg8ZgYgqeUJk2tMsZ5zi",
  "3": "6HQBWdC9BpY6Ky3YvmxoEJGce8PgQ6NqfkLgP3dJRDpb",
  "4": "FiDSNjCM5sLPmMvHvCWtnqEp9yt4NvZ2ThXJ9kpvBT73",
  "5": "4DwpsvxzemsHxDjz8qrVkQFP7nK5QwT6irL1KQkt5NJy",
  "6": "DBLeCRfF8t7q2oe5DYip8J9m4eRTc46EKMazqitth5eL",
  "7": "AUYNrra3XmPg4FBZ5t3w1m9Nou1AQqGEs5RUPqMu9Bz1",
  "8": "Ewj5hZ7Z2HLojk3Du7RXcHnNtqBv8UrMCNxuukHA436J",
  "9": "CJgszcJTg5MKFrRa9pcty42ybaaoq13thh3a71Ep2G8t",
  "10": "7zZKJ9g5Bz4UZnAjY9DoiGzsV56JSTbx3ejgrzuupKQ2",
  "11": "Fq1rT9krzj6zMUJbDUraMEekHKcTdwyfYeYdbcAgN5qE",
  "12": "EqnXBCwU2eTCVTE6G82PnZ5ujKpxeCnGrGvaURYMyjvr",
  "13": "4oWVGXtCMA7QQi8KJaWD2sc89kapQ9yYybTwqD99LVhi",
  "14": "3P7aumi3e4ZALiMY6g9Ec7H6zpN4jfyRC5Hs1G5n4Nt4",
  "15": "5pwz2Fn3YGFFLzNVFAZZESfpjjjnA2k71Hd2mHPCX1ro"
}
```

There is no direct way to find all print editions of a given master edition, so this command uses the following heuristic:

1. Derive the master edition account from the provided master_edition_mint
2. Decode the master edition NFT metadata account
3. Find the first verified creator of the master edition's metadata account
4. Snapshot all metadata accounts with the same first verified creator
5. Decode all the metadata accounts
6. Get the mint from each metadata account
7. Derive the edition for each mint account
8. Decode the edition and check if it's a master or a print
9. If it's a print and its parent is the master edition, add it to the list with its edition number as the key

The user can optionally pass in a first verified creator to use instead of deriving it from the master edition.

Limitations:

For master editions that have had prints minted, and then had their first verified creator changed and then more prints minted, it will take multiple runs of this command with each of the first verified creators to find all the prints and the user will need to track them in separate files and merge them manually as it will overwrite the default output file each time.

#### Usage

```bash
metaboss snapshot prints -m <MASTER_EDITION_MINT> -c <OPTIONAL_FIRST_VERIFIED_CREATOR>
````