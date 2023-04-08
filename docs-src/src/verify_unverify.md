## Verify

```
Verify Creators

USAGE:
    metaboss verify [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --log-level <log-level>    Log level [default: off]
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]

SUBCOMMANDS:
    creator        
    creator-all    
    help           Prints this message or the help of the given subcommand(s)
```

### Creator

Verify a creator in the metadata creators array by signing for it with its keypair. Creators can only verify themselves.


```
USAGE:
    metaboss verify creator [OPTIONS] --mint <mint>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --keypair <keypair>        Path to the update_authority keypair file
    -l, --log-level <log-level>    Log level [default: off]
    -a, --mint <mint>              Mint account of token to transfer
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss verify creator --account <MINT_ACCOUNT> --keypair <CREATOR_KEYPAIR_FILE>
```


### Creator All


Verify a creator in the metadata creators array of a list of metadata accounts, by signing for it with its keypair. Creators can only verify themselves.

```
USAGE:
    metaboss verify creator-all [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cache-file <cache-file>    Cache file
    -k, --keypair <keypair>          Path to the update_authority keypair file
    -l, --log-level <log-level>      Log level [default: off]
    -L, --mint-list <mint-list>      Mint list
    -R, --rate-limit <rate-limit>    Maximum number of requests per second [default: 10]
        --retries <retries>          Maximum retries: retry failed items up to this many times [default: 0]
    -r, --rpc <rpc>                  RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>          Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss verify creator-all --mint-list <MINT_LIST_FILE> --keypair <CREATOR_KEYPAIR_FILE>
```

## Unverify

```
Unverify Creators

USAGE:
    metaboss unverify [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --log-level <log-level>    Log level [default: off]
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]

SUBCOMMANDS:
    creator        
    creator-all    
    help           Prints this message or the help of the given subcommand(s)
```

### Creator

Unverify a creator in the metadata creators array by signing for it with its keypair. Creators can only unverify themselves.

```
USAGE:
    metaboss unverify creator [OPTIONS] --mint <mint>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --keypair <keypair>        Path to the update_authority keypair file
    -l, --log-level <log-level>    Log level [default: off]
    -a, --mint <mint>              Mint account of token to transfer
    -r, --rpc <rpc>                RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>        Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss unverify creator --account <MINT_ACCOUNT> --keypair <CREATOR_KEYPAIR_FILE>
```

### Creator All

Unverify a creator in the metadata creators array of a list of metadata accounts, by signing for it with its keypair. Creators can only unverify themselves.

```
USAGE:
    metaboss unverify creator-all [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --cache-file <cache-file>    Cache file
    -k, --keypair <keypair>          Path to the update_authority keypair file
    -l, --log-level <log-level>      Log level [default: off]
    -L, --mint-list <mint-list>      Mint list
    -R, --rate-limit <rate-limit>    Maximum number of requests per second [default: 10]
        --retries <retries>          Maximum retries: retry failed items up to this many times [default: 0]
    -r, --rpc <rpc>                  RPC endpoint url to override using the Solana config or the hard-coded default
    -T, --timeout <timeout>          Timeout to override default value of 90 seconds [default: 90]
```

#### Usage

```bash
metaboss unverify creator-all --mint-list <MINT_LIST_FILE> --keypair <CREATOR_KEYPAIR_FILE>
```