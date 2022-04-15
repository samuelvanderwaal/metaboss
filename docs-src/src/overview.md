# Metaboss

[![Stars](https://img.shields.io/github/stars/samuelvanderwaal/metaboss?style=social)](https://github.com/samuelvanderwaal/metaboss)
[![Forks](https://img.shields.io/github/forks/samuelvanderwaal/metaboss?style=social)](https://github.com/samuelvanderwaal/metaboss)
[![Crate](https://img.shields.io/crates/v/metaboss)](https://crates.io/crates/metaboss)
[![Downloads](https://img.shields.io/crates/d/metaboss)](https://crates.io/crates/metaboss)

The Solana Metaplex NFT 'Swiss Army Knife' tool.

Current supported features:

-   Decode the metadata of a token mint account

-   Mint new NFTs from a JSON file

-   Set `primary_sale_happened` bool on an NFT's metadata

-   Set `update_authorty` address on an NFT's metadata

-   Verify a creator by signing the metadata accounts for all tokens in a list, for a given candy machine id or a single mint account

-   Get a snapshot of current NFT holders for a given candy machine ID or update authority

-   Get a list of mint accounts for a given candy machine ID or update authority

-   Get a list of all candy machine state and config accounts for a given update authority

-   Update all metadata Data struct fields for a NFT

-   Update just the URI for a NFT

Suggestions and PRs welcome!

**Note: This is experimental software for a young ecosystem. Use at your own risk. The author is not responsible for misuse of the software or failing to test specific commands before using on production NFTs.**

## Binaries

Linux, MacOS and Windows binaries available in [releases](https://github.com/samuelvanderwaal/metaboss/releases), thanks to CI work done by [Kartik Soneji](https://github.com/KartikSoneji).

## Install From Source

Install [Rust](https://www.rust-lang.org/tools/install).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the source:

```bash
git clone git@github.com:samuelvanderwaal/metaboss.git
```

or

```bash
git clone https://github.com/samuelvanderwaal/metaboss.git
```

On Ubuntu you may need some additional packages:

```
sudo apt install libssl-dev libudev-dev pkg-config
```

Change directory and check out the `main` branch:

```bash
cd metaboss
git checkout main
```

Install or build with Rust:

```bash
cargo install --path ./
```

or

```bash
cargo build --release
```
