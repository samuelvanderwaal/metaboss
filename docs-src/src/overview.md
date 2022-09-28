# Metaboss

[![Stars](https://img.shields.io/github/stars/samuelvanderwaal/metaboss?style=social)](https://github.com/samuelvanderwaal/metaboss)
[![Forks](https://img.shields.io/github/forks/samuelvanderwaal/metaboss?style=social)](https://github.com/samuelvanderwaal/metaboss)
[![Crate](https://img.shields.io/crates/v/metaboss)](https://crates.io/crates/metaboss)
[![Downloads](https://img.shields.io/crates/d/metaboss)](https://crates.io/crates/metaboss)

The Solana Metaplex NFT 'Swiss Army Knife' tool.

Current Commands:

SUBCOMMANDS:
    burn            Full Burn a NFT
    burn-print      Full Burn a print edition NFT
    collections     NFT collections commands
    create          Create accounts
    decode          Decode on-chain data into JSON format
    derive          Derive PDAs for various account types
    find            Find things
    help            Prints this message or the help of the given subcommand(s)
    mint            Mint new NFTs from JSON files
    parse-errors    Parse Errors commands
    set             Set non-Data struct values for a NFT
    sign            Sign metadata for an unverified creator
    snapshot        Get snapshots of various blockchain states
    update          Update various aspects of NFTs
    uses            NFT uses commands

Each subcommand has additional commands. Run `metaboss <subcommand> --help` and `metaboss <subcommand> <command> --help` for more information on particular commands.

Suggestions and PRs welcome!

**Note: This is experimental software for a young ecosystem. Use at your own risk. The author is not responsible for misuse of the software or failing to test specific commands before using on production NFTs.**

**Test on devnet or localnet before using on mainnet.**
