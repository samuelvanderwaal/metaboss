## Burn

Burn Master Editions NFTs, as long as they do not have any editions (supply == 0). Only the owner/token holder of the NFT can burn it.

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

## Burn-Print

Burn Print Edition NFTs. Only the owner/token holder of the NFT can burn it.

### Burn-Print One

Fully burn a single Print Edition NFT by called the Token Metadata [burn_edition_nft](https://docs.metaplex.com/programs/token-metadata/instructions#burn-a-print-edition-nft) handler.

#### Usage

```bash
metaboss burn-print one -k <OWNER_KEYPAIR> --account <MINT_ACCOUNT>
```

### Burn-Print All

Fully burn multiple Print Edition NFTs by providing a JSON list file of mint accounts.

E.g. JSON file:

```json
[
    "D5ycm2mgBWDR37QVkvM389x84V4ux48bSeHLeiHPtX28",
    "4kYdMRRYtXjmkusgKEBntSXLDhqkHNE57GF3RPdtx6MW",
    "J8xuCFCeBRESoXewtMwrrpVUGikUG3B1WznNdLffyymz",
    "4gRtRjrbD7g5ZKUvSVA1tYMK9LZqz6uWuSc3rKeinySh"
]
```

Due to on-chain limitations, you also have to provide the mint account of the Master Edition NFT.

#### Usage

```bash
metaboss burn-print all -k <OWNER_KEYPAIR> -L <JSON_LIST_OF_MINT_ACCOUNTS> -m <MASTER_EDITION_MINT_ACCOUNT>
```

As in all other commands, keypair can be elided if set in the Solana config file.