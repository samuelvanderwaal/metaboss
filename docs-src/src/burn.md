## Burn

### Burn One

Fully burn a single NFT by calling the Token Metadata program [burn_nft](https://docs.metaplex.com/programs/token-metadata/instructions#burn-a-nft) handler.

#### Usage

```bash
metaboss burn one -k <OWNER_KEYPAIR> --account <MINT_ACCOUNT>
```

### Burn All

Fully burn multiple NFTs by providing a JSON list file of mint accounts.

E.g. JSON file:

```json
[
    "D5ycm2mgBWDR37QVkvM389x84V4ux48bSeHLeiHPtX28",
    "4kYdMRRYtXjmkusgKEBntSXLDhqkHNE57GF3RPdtx6MW",
    "J8xuCFCeBRESoXewtMwrrpVUGikUG3B1WznNdLffyymz",
    "4gRtRjrbD7g5ZKUvSVA1tYMK9LZqz6uWuSc3rKeinySh"
]
```

#### Usage

```bash
metaboss burn all -k <OWNER_KEYPAIR> -L <JSON_LIST_OF_MINTS_ACCOUNTS>
```

As in all other commands, keypair can be elided if set in the Solana config file.