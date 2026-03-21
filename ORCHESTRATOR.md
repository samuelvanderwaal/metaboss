# Metaboss Orchestrator Context

> Living document maintained by the orchestrator thread. Updated as the codebase evolves.
> This thread is the source of truth. The main thread orchestrates — subagents execute.

## Orchestration Protocol

**The main thread is the orchestrator. It does NOT implement changes directly.**

All non-orchestration work MUST be delegated to subagents. This includes:
- Writing or modifying code
- Adding tests
- Fixing bugs
- Updating CI/config files
- Creating new files

The orchestrator's responsibilities:
1. **Analyze** — understand the task, identify affected files, assess risk
2. **Delegate** — spawn subagent(s) with a clear prompt containing: goal, owned files, forbidden files, conventions, verification steps
3. **Review** — verify subagent output compiles and tests pass
4. **Integrate** — update this document, memory, and repo understanding
5. **Commit/PR** — only after orchestrator review and verification

When multiple independent tasks exist, spawn subagents in parallel. When tasks have dependencies, sequence them.

### Subagent Prompt Template

Every subagent prompt should include:
- **Goal**: What to accomplish (specific and measurable)
- **Files to create/modify**: Explicit list
- **Files NOT to touch**: Explicit list
- **Context**: Relevant architecture, output formats, patterns
- **Conventions**: `cargo fmt`, `anyhow::Result`, no unsafe, conventional commits
- **Verification**: What commands to run to confirm success (e.g., `cargo check --tests`, `cargo test --lib`)

## Project Identity

- **Name**: Metaboss — "The Metaplex NFT-standard Swiss Army Knife tool"
- **Language**: Rust (edition 2021), v0.45.0, Apache-2.0
- **Binary**: `metaboss` (CLI tool for Solana/Metaplex NFT operations)
- **Task runner**: `just` (see justfile at project root)

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

### Gitignore
- `*.json` is globally ignored — test fixture JSON files need `!tests/fixtures/**/*.json` exception in `.gitignore`

## CI/CD

- **CI** (`.github/workflows/ci.yml`): test + rustfmt + clippy + integration-test on ubuntu-latest
- **Build** (`.github/workflows/build.yml`): Cross-platform release binaries (Windows, Linux, macOS Intel+ARM)
- Triggers: push to main, PRs, tags `v*.*.*`
- Integration tests require Solana CLI v1.17.29, run with `--test-threads=1`

## Task Runner (justfile)

| Command | What it does |
|---------|-------------|
| `just test` | Run unit tests |
| `just integration-tests` | Run integration tests (requires solana-test-validator) |
| `just test-all` | Unit tests then integration tests |
| `just fmt` | cargo fmt |
| `just clippy` | cargo clippy -- -D warnings |
| `just check` | cargo check --tests |
| `just build` | cargo build |
| `just ci` | fmt check + clippy + unit tests |

## Tests

### Unit Tests (`just test`) — 130 total
- `setup.rs`: 27 tests (AppConfigBuilder + CliConfigBuilder, rate-limit mutex)
- `parse.rs`: 30 tests (creator_is_verified, parse_creators, parse_name, parse_symbol, parse_seller_fee_basis_points, is_only_one_option, parse_cli_creators, read_keypair)
- `utils.rs`: 22 tests (find_errors, find_tm_error, convert_to_wtf_error, generate_phf_map_var, clone_keypair)
- `mint.rs`: 13 tests (find_first_zero_bit bit manipulation)
- `data.rs`: 13 tests (Indexers FromStr/Display, round-trips, FoundError serde)
- `errors.rs`: 12 tests (Display output for all error variants, From conversions)
- `derive.rs`: 8 tests (PDA derivation)
- `cache/mod.rs`: 7 tests (Cache::new, write, update_errors, hex code extraction)

### Integration Tests (`just integration-tests`) — 28 total
- `tests/integration_tests.rs` (6): core NFT lifecycle (mint+decode, update uri+name, burn, transfer, sign, derive PDA)
- `tests/collection_tests.rs` (3): set-and-verify, verify/unverify lifecycle, set-size
- `tests/create_mint_tests.rs` (3): master edition + editions, mint --immutable, mint --sign
- `tests/decode_tests.rs` (5): mint-account, token-account, master edition, --full flag, raw bytes
- `tests/set_tests.rs` (4): set immutable, secondary-sale, update-authority, immutable-blocks-updates
- `tests/update_tests.rs` (4): update symbol, sfbp, creators, data
- `tests/verify_tests.rs` (3): verify creator, unverify creator, roundtrip

### Test Infrastructure
- `tests/common/mod.rs` — TestContext harness (starts validator, funds keypair, runs metaboss)
- `tests/fixtures/programs/` — 5 .so files (Token Metadata, Metaplex Core, SPL Token, Token-2022, ATA)
- `tests/fixtures/data/test_nft.json` — template NFT metadata (creator address filled at runtime)
- `scripts/refresh-programs.sh` — refreshes .so files from mainnet
- `.test_files/` — legacy test fixtures

### CLI Output Patterns (for integration tests)
- `mint one`: `Tx sig: "..." \nMint account: "..."` (Debug-formatted with quotes)
- `mint asset`: `Minted asset: "..." \nTransaction signature: "..."`
- `mint editions`: `Edition with mint: "..." \nCreated in tx: "..."`
- Update/set/burn commands: `Tx sig: "..."`
- Metadata fields are null-padded: always `trim_matches(char::from(0))`

### Legacy Tests
- `tests/mint.rs`: 3 ignored tests (pre-existing, manual validator setup)

## Current State

### Branch: `test/comprehensive-test-suite` (PR #375)
- Full test suite: 130 unit + 28 integration = 158 tests
- Test harness, fixtures, CI, justfile
- Flaky rate-limit test fixed
- setup.rs merge conflict cleaned up
- clippy::result_large_err suppressed for Rust 1.94+ compatibility
- `create fungible` tests excluded (requires program feature unavailable in test validator)

## Known Fragile Areas

1. **`opt.rs` (2005 lines)**: Massive file, any CLI change touches it. Tightly coupled to `process_subcommands.rs`.
2. **`process_subcommands.rs` (1278 lines)**: Giant match statement routing. Must stay in sync with `opt.rs`.
3. **Rate limiting globals**: `USE_RATE_LIMIT` and `RPC_DELAY_NS` are `lazy_static` `RwLock` globals in `constants.rs`. Tests use `RATE_LIMIT_TEST_MUTEX` in `setup.rs` to serialize access.
4. **Platform-specific keypair parsing**: `parse.rs` has `#[cfg(unix)]`/`#[cfg(windows)]`/`#[cfg(target_os = "macos")]` branches.
5. **External dependency on Solana config file**: Many operations silently fall back to defaults if `~/.config/solana/cli/config.yml` is missing.
6. **Integration test port**: All tests use port 8899 (solana-test-validator default). Must run with `--test-threads=1`.

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| (pre-existing) | Builder pattern for config | Cleaner than direct construction, testable |
| 2026-03-20 | Commit .so files to repo | Reliability over repo size; refresh script for updates |
| 2026-03-20 | One validator per test (TestContext) | Isolation over speed; tests are self-contained |
| 2026-03-20 | Integration tests use #[ignore] | Keep `cargo test` fast; run with --ignored flag |
| 2026-03-20 | Test via CLI binary, verify via metaboss_lib | Tests the real user path end-to-end |
| 2026-03-20 | Mutex for rate-limit tests | Global RwLock state leaks between parallel tests |
| 2026-03-20 | justfile for task running | Abstracts long cargo commands into memorable recipes |
| 2026-03-20 | Orchestrator delegates all implementation to subagents | Preserves main thread context; subagents get focused scope |
