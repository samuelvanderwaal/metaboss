[![Crate](https://img.shields.io/crates/v/metaboss)](https://crates.io/crates/metaboss)
[![Downloads](https://img.shields.io/crates/d/metaboss)](https://crates.io/crates/metaboss)
[![Build Status](https://img.shields.io/github/actions/workflow/status/samuelvanderwaal/metaboss/ci.yml?branch=main)](https://github.com/samuelvanderwaal/metaboss/actions)
[![License](https://img.shields.io/crates/l/metaboss)](https://github.com/samuelvanderwaal/metaboss/blob/main/LICENSE)

# Metaboss

![metaboss logo](mb_logo.gif?raw=true)

## Overview

A command-line tool for managing Solana NFTs and tokens via the Metaplex standard. Mint, update, transfer, burn, verify, decode, snapshot, and airdrop — for both legacy Token Metadata and Metaplex Core assets.

See the [docs](https://metaboss.dev) for full usage instructions. Suggestions and PRs welcome!

**Use at your own risk. Always test commands on devnet before running against production NFTs.**

## Installation

### Install Binary

Copy the following to a terminal:

```bash
bash <(curl -sSf https://raw.githubusercontent.com/samuelvanderwaal/metaboss/main/scripts/install.sh)
```

If you get errors you may need dependencies:

Ubuntu:

```bash
sudo apt install libudev-dev pkg-config
```

### Binaries

Linux and macOS binaries are available in [releases](https://github.com/samuelvanderwaal/metaboss/releases).

### Install From crates.io

```bash
cargo install metaboss
```

### Install From Source

Requires Rust 1.75 or later.

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

## Contact

Email: sam@vanderwaal.dev

Twitter: [@samvwaal](https://twitter.com/samvwaal)

Discord: @archaeopteryx#7615
