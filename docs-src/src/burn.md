## Burn

### Burn One

Burn a single NFT by burning the SPL token and closing the associated token account. If the keypair is also the NFT `update_authority` this will set the metadata account to blank/default values.

#### Usage

```bash
metaboss burn one -k <OWNER_KEYPAIR> --account <MINT_ACCOUNT>
```