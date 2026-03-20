# Metaboss Orchestrator Context

> Living document maintained by the orchestrator thread. Updated as the codebase evolves.

## Project Identity

- **Name**: Metaboss — "The Metaplex NFT-standard Swiss Army Knife tool"
- **Language**: Rust (edition 2021), v0.45.0, Apache-2.0
- **Binary**: `metaboss` (CLI tool for Solana/Metaplex NFT operations)

## Architecture

### Entry Flow
```
main.rs → Opt::from_args() (structopt)
        → AppConfigBuilder::new().rpc_url().timeout().build()
        → match Command variant → process_subcommands.rs → domain module
```

### Module Map (src/)

| Module | Type | Purpose |
|--------|------|---------|
| `main.rs` | Entry | CLI parsing, config init, command dispatch |
| `lib.rs` | Root | 30 pub mod declarations |
| `opt.rs` | CLI | ~2005 lines of structopt definitions, 21 top-level commands |
| `process_subcommands.rs` | Router | ~1278 lines, routes commands to domain modules |
| `setup.rs` | Config | `AppConfig`/`AppConfigBuilder`, `CliConfig`/`CliConfigBuilder` |
| `parse.rs` | Utils | Solana config parsing, keypair loading (JSON array + base58) |
| `constants.rs` | Config | Limits, program IDs, rate limit config |
| `errors.rs` | Types | `DecodeError`, `MigrateError`, `UpdateError`, `ActionError`, `SolConfigError` |
| `limiter.rs` | Infra | Token-bucket rate limiter for RPC calls |
| `spinner.rs` | UI | Progress bars/spinners (indicatif) |
| `cache/` | Infra | Batch operation result tracking, error caching |
| `data.rs` | Types | `Indexers` enum, `FoundError` struct |
| `utils.rs` | Infra | Transaction send/confirm, error code mapping |
| `wtf_errors.rs` | Data | Static error code → message maps (phf) |

### Domain Modules

| Module | Commands | Notes |
|--------|----------|-------|
| `airdrop/` | airdrop sol/spl | `sol.rs`, `spl.rs` |
| `burn/` | burn | `burn_asset.rs` (Core), `burn_legacy.rs` (legacy NFTs) |
| `check/` | check | Metadata validation |
| `collections/` | collections | `methods.rs`, `migrate.rs`, `items.rs`, `data.rs` |
| `create/` | create | `methods.rs` for metadata/fungible/editions |
| `decode/` | decode | On-chain data deserialization, `rule_set.rs` |
| `derive.rs` | derive | PDA derivation (8 variants) |
| `find.rs` | find | Missing edition finder |
| `mint.rs` | mint | ~1123 lines, batch minting, editions, fungibles |
| `sign.rs` | sign | Creator signature operations |
| `snapshot/` | snapshot | DAS API, indexer methods, print editions |
| `transfer/` | transfer | Asset transfers |
| `update/` | update | 12 update subcommands, one file per property |
| `uses.rs` | uses | Use authority management |
| `verify/` | verify | Creator verification |
| `unverify/` | unverify | Creator unverification |
| `theindexio/` | (internal) | TheIndex.io API integration |
| `extend_program.rs` | extend-program | Program account space extension |

## Key Dependencies

- **Solana**: solana-client/sdk/program ~1.17.29
- **Metaplex**: mpl-token-metadata 4.1.2, mpl-core 0.7.0
- **SPL**: spl-token 3.5.0, spl-token-2022 1.0.0
- **CLI**: structopt 0.3.26 (not clap directly)
- **Async**: tokio 1.35.1
- **Parallelism**: rayon 1.8.0
- **HTTP**: reqwest 0.11.23
- **Errors**: anyhow + thiserror
- **External lib**: metaboss_lib 0.23.0

## Conventions

### Code Style
- `cargo fmt` before every commit (per user's CLAUDE.md)
- Clippy with `-D warnings` in CI
- No custom rustfmt.toml or clippy.toml
- No unsafe code anywhere
- `anyhow::Result<T>` for all fallible functions

### Commit Messages
- Conventional commits: `type: description`
- Types: `feat`, `fix`, `refactor`, `test`, `chore`
- PR references: `(#NNN)`
- Co-author line: `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>`

### Naming
- Handler functions: `process_{command}()`
- Builder pattern: `FooBuilder::new().field().build() -> Result<Foo>`
- Subcommand enums: `{Command}Subcommands`

### Config Resolution Priority
1. CLI arguments (`--rpc`, `-T`)
2. Solana CLI config (`~/.config/solana/cli/config.yml`)
3. Defaults (devnet, 90s timeout)

## CI/CD

- **CI** (`.github/workflows/ci.yml`): test + rustfmt + clippy on ubuntu-latest
- **Build** (`.github/workflows/build.yml`): Cross-platform release binaries (Windows, Linux, macOS Intel+ARM)
- Triggers: push to main, PRs, tags `v*.*.*`

## Tests

### Unit Tests (cargo test --lib)
- `setup.rs`: 25 tests (AppConfigBuilder + CliConfigBuilder)
- `derive.rs`: 8 tests (PDA derivation)
- `parse.rs`: 30 tests (creator_is_verified, parse_creators, parse_name, parse_symbol, parse_seller_fee_basis_points, is_only_one_option, parse_cli_creators, read_keypair)
- `cache/mod.rs`: 7 tests (Cache::new, write, update_errors, hex code extraction)
- **Total: 70 unit tests**

### Integration Tests (cargo test --test integration_tests -- --ignored --test-threads=1)
- `tests/integration_tests.rs`: 6 tests covering core NFT lifecycle
  - `test_mint_one_and_decode` — mint + decode + verify metadata
  - `test_mint_update_uri_and_name` — mint + update URI + update name + verify
  - `test_mint_and_burn` — mint + burn + verify token gone
  - `test_mint_and_transfer` — mint + transfer + verify balances
  - `test_mint_and_sign` — mint + sign + verify creator verified
  - `test_derive_metadata_pda` — derive PDA via CLI + verify against library

### Test Infrastructure
- `tests/common/mod.rs` — TestContext harness (starts validator, funds keypair, runs metaboss)
- `tests/fixtures/programs/` — 5 .so files (Token Metadata, Metaplex Core, SPL Token, Token-2022, ATA)
- `tests/fixtures/data/test_nft.json` — template NFT metadata
- `scripts/refresh-programs.sh` — refreshes .so files from mainnet
- `.test_files/` — legacy test fixtures

### CI Integration
- CI runs unit tests in `test` job, integration tests in `integration-test` job (with Solana CLI v1.17.29)

### Legacy Tests
- `tests/mint.rs`: 3 ignored tests (pre-existing, manual validator setup)

## Current State

### Branch: `refactor/config-builder`
- Introduced `AppConfigBuilder` and `CliConfigBuilder` in `setup.rs`
- Builder pattern replaces direct config construction in `main.rs`
- Comprehensive tests added for both builders
- `setup.rs` is modified (staged)

## Known Fragile Areas

1. **`opt.rs` (2005 lines)**: Massive file, any CLI change touches it. Tightly coupled to `process_subcommands.rs`.
2. **`process_subcommands.rs` (1278 lines)**: Giant match statement routing. Must stay in sync with `opt.rs`.
3. **Rate limiting globals**: `USE_RATE_LIMIT` and `RPC_DELAY_NS` are `lazy_static` `RwLock` globals in `constants.rs`. Tests that touch rate limiting can interfere with each other.
4. **Platform-specific keypair parsing**: `parse.rs` has `#[cfg(unix)]`/`#[cfg(windows)]`/`#[cfg(target_os = "macos")]` branches.
5. **External dependency on Solana config file**: Many operations silently fall back to defaults if `~/.config/solana/cli/config.yml` is missing.

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| (pre-existing) | Builder pattern for config | Cleaner than direct construction, testable |
| 2026-03-20 | Commit .so files to repo | Reliability over repo size; refresh script for updates |
| 2026-03-20 | One validator per test (TestContext) | Isolation over speed; tests are self-contained |
| 2026-03-20 | Integration tests use #[ignore] | Keep `cargo test` fast; run with --ignored flag |
| 2026-03-20 | Test via CLI binary, verify via metaboss_lib | Tests the real user path end-to-end |

## Subagent Guidelines

When spawning subagents for this repo:
- **Must run**: `cargo fmt` before any commit
- **Must not touch**: Files outside their assigned scope
- **Must verify**: `cargo test` passes, `cargo clippy -- -D warnings` clean
- **Convention**: Use `anyhow::Result`, conventional commits, no unsafe code
- **Large files warning**: `opt.rs` and `process_subcommands.rs` are tightly coupled — changes to one usually require changes to the other
