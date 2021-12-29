v0.3.4
* Added support for v2 candy machine ids for `sign all`

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