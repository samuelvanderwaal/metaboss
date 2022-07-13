## Set

**Warning: These commands modify your NFT and are for advanced users. Use with caution.**

Set non-Data struct values for a NFT.

### Set Secondary Sale

Set `primary_sale_happened` to be `true`, enabling secondary sale royalties.

```bash
metaboss set secondary-sale --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT>
```

Outputs a TxId to the command line so you can check the result.

### Set Secondary Sale All

Same as `set secondary-sale` but takes a mint list instead of a single account file.

### Set Update-Authority

Set `update_authority` to a different public key.

```bash
metaboss set update-authority --keypair <PATH_TO_KEYPAIR> --account <MINT_ACCOUNT> --new-update-authority <NEW_UPDATE_AUTHORITY>
```

### Set Update-Authority-All

Set `update_authority` to a different public key for a list of NFTs.

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