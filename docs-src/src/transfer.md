## Transfer

Transfer Metaplex assets such as NFTs and pNFTs.

### Transfer Asset

```
USAGE:
    metaboss transfer asset [OPTIONS] --mint <mint> --receiver <receiver>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --amount <amount>          Amount of tokens to transfer, for NonFungible types this must be 1 [default: 1]
    -k, --keypair <keypair>        Path to the update_authority keypair file
    -l, --log-level <log-level>    Log level [default: off]
    -m, --mint <mint>              Mint account of token to transfer
    -R, --receiver <receiver>      Receiving address, if different from update authority
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]
```

### Usage

```bash
metaboss transfer asset --mint <asset_mint_address> --receiver <receiver_wallet_address>
```

E.g.:

```bash
metaboss transfer asset --mint 2KGQLgypChErw3kKPqG26uyUjVtZj8QSJg2AUNR7BWdM -R PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8
```

**Amount**

For non-fungible types such as `NonFungible` and `ProgrammableNonFungible`, the amount can only be `1` and that is the default value for the CLI argument in not specified. 

For fungible types, specify the amount to be transferred with `--amount <number>`.