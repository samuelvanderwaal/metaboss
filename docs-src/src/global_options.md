## Global Options

These are the options that apply to all subcommands and can be passed in at any level.

```bash
metaboss <option> <subcommand> <subcommand>
```

```bash
metaboss <subcommand> <option> <subcommand>
```

```bash
metaboss <subcommand> <subcommand> <option>
```

## Options

-r, --rpc <rpc> The RPC endpoint to use for commands.

Metaboss will try to read your Solana config settings for both the RPC endpoint and also the Commitment setting by reading from `$HOME/.config/solana/cli/config.yml`. If it can't find a config file it defaults to using `https://dev.genesysgo.net` and `confirmed`.

Running Metaboss with the `--rpc` option will override the above with whatever RPC endpoint the user provides.

-T, --timeout <timeout> The timeout in seconds to use for RPC calls.

This defaults to 90 seconds which should be fine for most cases but can be overriden if needed.

Example:

```bash
metaboss snapshot holders -r https://ssc-dao.genesysgo.net/ -T 120 -u DC2mkgwhy56w3viNtHDjJQmc7SGu2QX785bS4aexojwX
```