## Snapshot

Get snapshots of various blockchain states.

**Note**: Most of the snapshot commands rely on the [Digital Asset Standard (DAS) API](https://developers.metaplex.com/bubblegum#metaplex-das-api), which is a read layer for Metaplex NFTs that uses indexed data to serve up information without having to make onerous getProgramAccounts RPC calls to validators. To use these commands you will need to have a RPC URL set with a provider that supports the DAS API. The current official list from Metaplex is [here](https://developers.metaplex.com/bubblegum/rpcs).

Metaboss recommends using [Helius](https://helius.dev) for DAS API calls as they are the only provider that fully supported the DAS API spec on both mainnet and devnet when these commands were tested. In addition, they have a very generous free tier that should be sufficient for most casual users.

### Snapshot Holders-GPA
(Legacy: not recommended for use.)

Snapshot all current holders of NFTs using the legacy getProgramAccounts method, filtered by verified candy_machine_id/first creator or update_authority.
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

* owner -- the address of the holder of the token
* ata -- the associated  token address the NFT is stored at
* mint -- the token mint address for the NFT
* metadata-- the address of the metadata decorating the mint account that defines the NFT

Example file:

```json
[
    {
        "owner": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB",
        "ata": "7yGA66LYDU7uoPW2x9jrUKaDWTs9jqZ5cSNKR1VaLQdw",
        "mint": "C2eGm8iQPnKVWxakyo8QhwJUvYrZHKF52DPQuAejpTWG",
        "metadata": "8WTA3sLxwRNDKHxZFbn2CFo3FX1ZP59EqrvuDPLbmmWV"
    }
]
```

### Snapshot Mints-GPA
(Legacy: not recommended for use.)

Snapshot all mint accounts using the legacy getProgramAccounts method, for a given verified candy machine id/first creator or update authority

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

### Snapshot Holders -- DAS API

Snapshot all current holders by various group types:

- Mint
Gets all token holders of a specific mint (unimplemented--not supported in DAS yet).

- First Verified Creator Address (FVCA)
Gets all holders of NFTs with a specific FVCA.

- Metaplex Collection Id (MCC)
Gets all holders of NFTs with a specific verified MCC ID.

#### Usage

```bash
metaboss snapshot holders <GROUP_VALUE> --group-key <GROUP_KEY>
```

Creates a JSON file in the output directory with the name format of `<GROUP_VALUE>_<GROUP_KEY>_holders.json` consisting of an array of objects with the following fields:

* owner -- the address of the holder of the token
* ata -- the associated  token address the NFT is stored at
* mint -- the token mint address for the NFT
* metadata-- the address of the metadata decorating the mint account that defines the NFT

E.g.:

```json
  {
    "owner": "42NevAWA6A8m9prDvZRUYReQmhNC3NtSZQNFUppPJDRB",
    "mint": "2pwsTyuM4Cb2zmN3xydti2ysPYdmu242w1J7TmQya3At",
    "metadata": "Art3NUzP2DxqfzwwMgjLdu8KY9NQLBp2zuEZ63dx9iU2",
    "ata": "FfwoNCYYC5wUkTYTmtYmBSgk9YRWpTTWZCpJB6MjwvSk"
  },
```

Example command:

```bash
metaboss snapshot holders PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8 -g fvca
```

### Snapshot Mints -- DAS API

Snapshot all mint accounts by various group types:

- Authority
Gets all NFT mint addresses for a given update authority. 
**Warning:** update authority can be set to any address so use this option with caution.

- Creator
Gets all NFT mint addresses that have a specific verified creator.

- Metaplex Collection Id (MCC)
Gets all NFT mint addresses with a specific verified MCC ID.

#### Usage

```bash
metaboss snapshot mints <GROUP_VALUE> --group-key <GROUP_KEY>
```

Creates a JSON file in the output directory with the name format of `<GROUP_VALUE>_<GROUP_KEY>_mints.json` consisting of an array of mint accounts.

For the creator method you can optionally specify which creator position to use with the `--position` option. This defaults to 0, which is the first verified creator in the creators array.

Example command:

```bash
metaboss snapshot mints PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8 -g creator --position 1
```

### Snapshot FVCA -- DAS API

An alias for snapshot mints with the group key set to `creator` and the default position of 0 used to find all mints with a given FVCA.

#### Usage

```bash
metaboss snapshot fvca <FVCA>
```

Creates a JSON file in the output directory with the name format of `<FVCA>_fvca_mints.json` consisting of an array of mint accounts.

Example command:

```bash
metaboss snapshot fvca PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8
```

### Snapshot MCC -- DAS API

An alias for snapshot mints with the group key set to `mcc` used to find all mints with a given MCC.

#### Usage

```bash
metaboss snapshot mcc <MCC>
```

Creates a JSON file in the output directory with the name format of `<MCC>_mcc_mints.json` consisting of an array of mint accounts.

Example command:

```bash
metaboss snapshot mcc PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8
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