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