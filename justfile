# Default recipe: list available commands
default:
    @just --list

# Run unit tests
test:
    cargo test --lib

# Run integration tests (requires solana-test-validator)
integration-tests:
    cargo test --test integration_tests -- --ignored --test-threads=1

# Format code
fmt:
    cargo fmt

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# Build the project
build:
    cargo build

# Fast compilation check including test code
check:
    cargo check --tests

# Run all tests: unit then integration
test-all: test integration-tests

# CI pipeline: format check, clippy, unit tests
ci:
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
    cargo test --lib
