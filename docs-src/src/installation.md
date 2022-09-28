# Installation

## Recommended Way to Install

For non-technical users, the recommended way to install is to use the install script to install on **MacOs**, **Ubuntu**, or other **Unix-like OS**, by copying the following into a terminal:

```
bash <(curl -sSf https://raw.githubusercontent.com/samuelvanderwaal/metaboss/main/scripts/install.sh)
```

This will download the appropriate binary for your system and install it. Feel free to inspect the install script directly at [the link](https://raw.githubusercontent.com/samuelvanderwaal/metaboss/main/scripts/install.sh) to see what you are running before you run the command.

For **Windows**, either use the prebuilt binary in the following section, or install Windows Subsystem Linux (WSL) to use the Ubuntu terminal to run the above installation script.

To install WSL on Windows, either run `wsl --install -d ubuntu` in the cmd prompt terminal, or install "Ubuntu" from the Windows app store. Once you have that set up you can simply run the install script in the WSL terminal. You will then run all your Metaboss commands from WSL as well.

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

## Set Up Your Solana Config

If you have the [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) installed (recommended) you can setup your RPC and keypair so you don't have to pass them into Metaboss:

```
solana config set --url <rpc url> --keypair <path to keypair file>
```