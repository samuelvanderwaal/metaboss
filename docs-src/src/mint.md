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