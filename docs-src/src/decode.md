## Decode

### Decode Mint

Decodes a mint account's metadata into a JSON file. It accepts either a single account or a list of accounts.

#### Usage

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

As of v0.4.0, the default output will only be the `Data` struct matching the input format of the `update data` and `update data-all` commands. To get the full `Metadata` struct, use the `--full` option.

Use `--raw` to get the account data as raw bytes for debugging purposes.

### Decode Edition

Decodes a single Print Edition account from a mint account into a JSON file. This is a Print Edition PDA.

#### Usage

```bash
metaboss decode edition --account <MINT_ACCOUNT> 
```

### Decode Edition Marker

Decodes a single Edition Marker PDA account from a mint account into a JSON file. This takes the Master Edition NFT mint account and either the edition number or the desired edition marker number, zero-indexed.

#### Usage

In this example, it will decode the 2nd Edition Marker PDA which corresponds to Edition numbers 248-495.

```bash
metaboss decode edition-marker --account <MASTER_EDITION_MINT_ACCOUNT> -m 1
```
### Decode Master

Decodes a single Master Edition account from a mint account into a JSON file. This is a Master Edition PDA.

#### Usage

```bash
metaboss decode master --account <MINT_ACCOUNT> 
```

### Decode Rulset

Decode a programmable NFT rule set from a pubkey.

#### Usage

```bash
metaboss decode rule-set AdH2Utn6Fus15ZhtenW4hZBQnvtLgM1YCW2MfVp7pYS5
```

### Decode Pubkey

Decode a pubkey from a u8 array.

#### Usage

```bash
metaboss decode pubkey "[198,63,89,223,232,36,128,201,194,84,163,124,239,91,140,18,189,137,137,47,53,111,44,226,53,45,91,202,241,224,183,205]"
```