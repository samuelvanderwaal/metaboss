## Mint

Mint new NFTs from JSON files. 

For both subcommands the `--immutable` flag sets the NFT data to be immutable and the `--primary-sale-happened` flag sets the primary sale happened bool to true.

### Mint One

Mint a single NFT from a JSON file.

#### Usage

```bash
metaboss mint one --keypair <KEYPAIR> --nft-data-file <PATH_TO_NFT_DATA_FILE> --receiver <RECEIVER_ADDRESS>
```

```bash
metaboss mint one --keypair <KEYPAIR> --external-metadata-uri <EXTERNAL_METADATA_URI> --receiver <RECEIVER_ADDRESS> --immutable --primary-sale-happened
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

The --external-metadata-uri option takes a URI to an external metadata file such as an Arweave link pointing to a JSON file.

If `receiver` is set, the NFT will be minted directly to the receiver's address, otherwise it is minted to `keypair`. Observant users may note that with a simple bash script this allows airdrops to be deployed with Metaboss.

Use the `--sign` option to sign the metadata with the keypair immediately after minting.

#### Editions

To mint a NFT with the ability to print editions from it use the `--max-editions <max-editions>` option. This defaults to `0` meaning no editions are allowed. Setting it to a positive integer means you can print up to that many editions. Setting to a value of `-1` means unlimited editions. Because of how the CLI interprets the `-` symbol to set max editions to infinite you should use the `=` sign for the `--max-editions` option: `metaboss mint one -a <master_account> --max-editions='-1'`.

To mint editions from a master NFT use the`metaboss mint editions` command to either mint the next `n` editions sequentially using `--next-editions <int>` or mint specific edition numbers using `--specific-editions <int> <int> <int>` with a list of integer edition numbers to mint.

To find any edition numbers in the sequence that have not been minted use `metaboss find missing-editions`.

To find and mint any missing editions and mint them to the authority keypair use `metaboss mint missing-editions`.

To find the full list of options for each command use `-h` or `--help` as normal.


### Mint List

Mint multiple NFTs from a list of JSON files.


#### Usage

```bash
metaboss mint list --keypair <KEYPAIR> --nft-data-dir <PATH_TO_NFT_DATA_FILE> --receiver <RECEIVER_ADDRESS>
```
This command functions the same as `mint one` except instead of a single JSON file, provide a path to a directory with multiple JSON files, one for each NFT to be minted.

```bash
metaboss mint list --keypair <KEYPAIR> --external-metadata-uris <PATH_TO_JSON_FILE> --receiver <RECEIVER_ADDRESS> --immutable --primary-sale-happened
```

To mint from URIs provide the path to a JSON file containing a list of URIs.

By default, new NFTs are minted as mutable, to make them immutable use the `--immutable` option.

Use the `--sign` option to sign the metadata with the keypair immediately after minting.


### Mint Asset

Mint various types of Metaplex assets, including pNFTs.


```
USAGE:
    metaboss mint asset [OPTIONS] --asset-data <asset-data>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --amount <amount>
            Amount of tokens to mint, for NonFungible types this must be 1 [default: 1]

    -d, --asset-data <asset-data>                                Asset data
        --decimals <decimals>                                    Mint decimals for fungible tokens
    -k, --keypair <keypair>                                      Path to the update_authority keypair file
    -l, --log-level <log-level>                                  Log level [default: off]
    -s, --max-print-edition-supply <max-print-edition-supply>
            Max supply of print editions. Only applies to NonFungible types. 0 for no prints, n for n prints,
            'unlimited' for unlimited prints
    -R, --receiver <receiver>                                    Receiving address, if different from update authority
    -r, --rpc <rpc>
            RPC endpoint url to override using the Solana config or the hard-coded default

    -T, --timeout <timeout>
            Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

You need an asset json file of this format:

```json
    {
        "name": "Studious Crab #1",
        "symbol": "CRAB",
        "uri": "https://arweave.net/uVtABL4PYv0wVke3LL4DLMkqkSMcQl1qswRZNkJ0a0g",
        "seller_fee_basis_points": 100,
        "creators": [
            {
                "address": "ccc9XfyEMh9sU6DRkUmqQGJqgdKb6QyUaaT5h5BGYw4",
                "verified": true,
                "share": 100
            }
        ],
        "primary_sale_happened": false,
        "is_mutable": true,
        "token_standard": "ProgrammableNonFungible",
        "collection": null,
        "uses": null,
        "collection_details": null,
        "rule_set": null
    }
```

Substitute appropriate values for each field. The creator can only be set as verified if it is the same keypair as the one used to mint the asset, otherwise leave it as `false`.


```bash
metaboss mint asset -d <asset_json_file> -k <keypair> -R <receiver> -s <print_supply>
```

E.g.:

```bash
metaboss mint asset -d crab.json -k ccc9XfyEMh9sU6DRkUmqQGJqgdKb6QyUaaT5h5BGYw4.json -R  PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8 -s 0
```

Leave off the `--receiver` option to mint to your keypair.

**Print Supply**

All non-fungible type assets: currently `NonFungible` and `ProgrammableNonFungible`, require the `print-supply` option to be specified to set the maximum number of print editions that can be minted from the asset. For most PFP, 1/1, style NFTs, this should be set to `0` to prevent any editions being minted. Other options are: `n` for a limited number of `n` editions (e.g. `10`), or `unlimited` to allow unlimited editions to be minted.

Fungible types such as `Fungible` and `FungibleAsset` should leave this value off as it has no meaning for them and the `mint asset` command will fail if that is specified for a fungible type.
