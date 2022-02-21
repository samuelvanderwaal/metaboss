v0.4.2
* Update metadata by name, symbol & creator
* Add `sign` option to mint commands
* Change rate limiting for GenesysGo and allow custom rate limits

v0.4.1
* Hot fix: add rate-limiting to all par_iter functions

v0.4.0
* Change decode default format to data struct to match input required from `update-metadata` command
* Add `set immutable` and `set immutable` all commands
* Updated to use `mpl-token-metadata v1.1.0`
* GenesysGo mainnet RPC node to list of public nodes for rate-limiting

v0.3.7
* Removes string interpolation to be compatible with versions of Rust lower than 1.58

v0.3.6
* Add withdraw command for candy machine v2

v0.3.5
* Make timeout, rpc and log-level global options.
* Increase default timeout to 90 seconds to accommodate longer `getProgramAccount` calls.
* Default to GenesysGo devnet node instead of failing if no rpc provided and no config file found.

v0.3.4
* Added support for v2 candy machine ids for `sign all`
* Add `burn one` function for burning NFT SPL token and clearing Metadata account.

v0.3.3

* Added exponential backoff retries to network requests: 250 ms, 500 ms, 1000 ms then fails.
* Added support for snapshot mints and holders commands for v2 candy machine ids.
* Added `derive` subcommand for deriving PDAs.

v0.3.2

* Check first creator is verified in snapshot mints and snapshot holders commands.


v0.3.1

* Add `primary_sale_happened` flag to mint commands
* Add ability to mint new tokens from URI instead of JSON file
* Fixed bug where RPC url was not accepted if there was no config file
* Removed progress bars from minting commands as they didn't work