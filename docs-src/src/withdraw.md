## Withdraw

### CM-V2

Withdraw all funds from a candy machine v2 account. 

**Warning: This command will drain your candy machine account and cause it to be garbage collected. Any unminted NFT items will be lost.**

#### Usage

```bash
metaboss withdraw cm-v2 <CANDY_MACHINE_ID> -k <PATH_TO_KEYPAIR>
```

Example:

```bash
metaboss withdraw cm-v2 C2eGm8iQPnKVWxakyo8QhwJUvYrZHKF52DPQuAejpTWG -k ./keypair.json
```