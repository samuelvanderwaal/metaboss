## Set

**Warning: These commands modify your NFT and are for advanced users. Use with caution. 
Set commands are either irreversible or require a new update authority to reverse.**

Set non-Data struct values for a NFT.

### Set Secondary Sale 

Set `primary_sale_happened` to be `true`, enabling secondary sale royalties. **This is not reversible.**

```bash
metaboss set secondary-sale --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT>
```

Outputs a TxId to the command line so you can check the result.

### Set Secondary Sale All

Same as `set secondary-sale` but takes a mint list instead of a single account file. **This is not reversible.**

### Set Update-Authority

Set `update_authority` to a different public key. **This is not reversible by the original update authority.**

```bash
metaboss set update-authority --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-update-authority <NEW_UPDATE_AUTHORITY>
```

### Set Update-Authority-All

Set `update_authority` to a different public key for a list of NFTs. **This is not reversible by the original update authority.**

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

### Set Immutable

Set an NFT's `Data` struct to be immutable. **This is not reversible.**

```bash
metaboss set immutable --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT>
```

### Set Immutable-All

Set all NFTs in a list to be immutable. **This is not reversible.**

```bash
metaboss set immutable-all --keypair <PATH_TO_KEYPAIR> --mint-accounts-file <PATH_TO_MINT_ACCOUNTS>
```

### Set Token Standard

Set an asset's Token Standard to automatically be the correct type. **This is not reversible.**

```
USAGE:
    metaboss set token-standard [OPTIONS] --account <account>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account <account>        Mint account of corresponding metadata to update
    -k, --keypair <keypair>        Path to the update authority's keypair file
    -l, --log-level <log-level>    Log level [default: off]
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss set token-standard --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT>
```

### Set Token Standard-All

Set all assets in a list to be the correct Token Standard. **This is not reversible.**

```
USAGE:
    metaboss set token-standard-all [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cache-file <cache-file>    Cache file
    -k, --keypair <keypair>          Path to the update authority's keypair file
    -l, --log-level <log-level>      Log level [default: off]
    -L, --mint-list <mint-list>      Mint list
    -R, --rate-limit <rate-limit>    Maximum number of requests per second [default: 10]
        --retries <retries>          Maximum retries: retry failed items up to this many times [default: 0]
    -r, --rpc <rpc>                  RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>          Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss set token-standard-all --keypair <PATH_TO_KEYPAIR> --mint-list <PATH_TO_MINT_ACCOUNTS>
```