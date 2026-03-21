#!/usr/bin/env bash
# Dumps program .so files from mainnet for use in integration tests.
# Idempotent: re-running overwrites existing files.

set -euo pipefail

FIXTURES_DIR="tests/fixtures/programs"
RPC_URL="mainnet-beta"

mkdir -p "$FIXTURES_DIR"

declare -A PROGRAMS=(
    ["metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"]="token_metadata.so"
    ["CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"]="mpl_core.so"
    ["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"]="spl_token.so"
    ["TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"]="spl_token_2022.so"
    ["ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"]="associated_token_account.so"
)

for address in "${!PROGRAMS[@]}"; do
    filename="${PROGRAMS[$address]}"
    dest="$FIXTURES_DIR/$filename"
    echo "Dumping $filename ($address) -> $dest"
    solana program dump -u "$RPC_URL" "$address" "$dest"
done

echo "Done. All programs saved to $FIXTURES_DIR/"
