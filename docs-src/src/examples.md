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