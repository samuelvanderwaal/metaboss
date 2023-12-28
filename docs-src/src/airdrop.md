## Airdrop

This is an experimental feature that uses the TPU client to rapidly and efficiently make transfers. It relies on the [Jib library](https://github.com/samuelvanderwaal/jib) for transmitting instructions via TPU. **You should carefully test it on devnet prior to running it on mainnet.**

The benefit of using the TPU client is that is can rapidly transmit the transactions directly to the consensus leader and the command does not require a high-throughput private RPC node and can use the public ones by default, as the RPC node is only used for determining the current leader.

### Airdrop SOL

Airdrop SOL to a list of accounts. 



```
Airdrop SOL

USAGE:
    metaboss airdrop sol [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cache-file <cache-file>            Cache file
    -k, --keypair <keypair>                  Path to the owner keypair file
    -l, --log-level <log-level>              Log level [default: off]
    -n, --network <network>                  Network cluster to use, defaults to devnet [default: devnet]
    -L, --recipient-list <recipient-list>    Path to the mint list file
    -r, --rpc <rpc>                          RPC endpoint url to override using the Solana config or the hard-coded
                                             default
    -T, --timeout <timeout>                  Timeout to override default value of 90 seconds [default: 90]
```

This command requires a recipient list file that should be a hashmap/dictionary/object of addresses and the amount of lamports to send to them. (1 SOL = 1,000,000,000 lamports) E.g.:

```json
{
  "HVtodaLcq6zVvqp7h6JwLLrsAGxeJ9BatvgpUfp9b4oM": 1000,
  "5VXU4QbhUZbkBqKxT3Mv55krE4MomMgtV68whNRotjk5": 1000,
  "GSFKDFeCe93aUscmG84ugtXXNPMGoMcbZwRaamPLXS9o": 5000,
  "DCYHBcWGgdUUCBbj7rjbkBJWkuoHAH88BzMKfbbkFUNJ": 7000,
  "8MUCm4HxRXQUKMyanyNcvcG4qbAmw5s6y9exiszFZgg": 5000,
  "sknqbvGgVFpniWRK9kM1e77Fuq5oEhSZ5He4PtbTeZh": 3000  
}
```



#### Usage

```bash
metaboss airdrop sol  -L <PATH_TO_RECIPIENTS_LIST_FILE> -n devnet
```

This command creates two files: `mb-cache-airdrop-<TIMESTAMP>.json` and `mb-successful-airdrops-<TIMESTAMP>.json`. The cache file is used to track the airdrop progress by storing failed transactions and the successful airdrops file is used to track the successful airdrops by storing transaction signatures of the successful airdrops. 

To re-run failed transactions run the command with the cache file instead of the recipient list file:

```bash
metaboss airdrop sol  -c <PATH_TO_CACHE_FILE> -n devnet
```

If transactions continuously fail you should look at the errors in the cache file and determine the cause.

### Airdrop SPL Tokens

Airdrop SPL tokens to a list of accounts.

```bash
Airdrop SPL tokens

USAGE:
    metaboss airdrop spl [FLAGS] [OPTIONS] --mint <mint>

FLAGS:
        --boost          Boost the transactions w/ priority fees
    -h, --help           Prints help information
        --mint-tokens    
    -V, --version        Prints version information

OPTIONS:
    -c, --cache-file <cache-file>            Cache file
    -k, --keypair <keypair>                  Path to the owner keypair file
    -l, --log-level <log-level>              Log level [default: off]
    -m, --mint <mint>                        Mint from the SPL token mint
    -n, --network <network>                  Network cluster to use, defaults to devnet [default: devnet]
    -L, --recipient-list <recipient-list>    Path to the mint list file
    -r, --rpc <rpc>                          RPC endpoint url to override using the Solana config or the hard-coded
                                             default
    -T, --timeout <timeout>                  Timeout to override default value of 90 seconds [default: 90]
```

This command works similarly to the SOL airdrop command, but expects the amount to be in the display units of the SPL token. E.g. for a token with three decimal places the amount of 10 will be converted to 10,000 base units behind the scenes.

Be aware that airdropping SPL tokens to wallets that do not already have a token account for that mint will cost 0.002 SOL per transaction. This is because the token account needs to be created first. This could end up being a significant cost if you are airdropping to a large number of wallets. 

For large SPL token airdrops you may want to consider setting up a claim site instead.