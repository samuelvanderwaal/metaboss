# Metaboss

Metaboss is a Solana Metaplex NFT CLI tool ("Swiss Army Knife"). It is a single Rust binary crate — not a monorepo.

## Cursor Cloud specific instructions

### Development commands

All standard dev commands match CI (see `.github/workflows/ci.yml`):

- **Build:** `cargo build`
- **Lint (format):** `cargo fmt --all -- --check`
- **Lint (clippy):** `cargo clippy -- -D warnings`
- **Unit tests:** `cargo test`

### System dependencies

Building requires `libssl-dev`, `libudev-dev`, and `pkg-config` (Ubuntu). These are pre-installed in the VM snapshot.

### Integration tests

The 3 integration tests in `tests/mint.rs` are `#[ignore]` by default. They require:
1. `solana-test-validator` running on `localhost:8899`
2. The `metaboss` binary on `PATH`

These are intentionally excluded from `cargo test` and CI. Run them manually with `cargo test -- --ignored` only after setting up a local validator.

### Running the CLI

Use `cargo run -- <subcommand>` for development. The `derive` subcommands (e.g. `cargo run -- derive metadata <MINT_ACCOUNT>`) work offline and are useful for quick smoke tests. Commands that interact with the Solana blockchain (e.g. `decode`, `mint`, `update`) need a `--rpc` endpoint or a configured Solana CLI default.
