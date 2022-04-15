## Decode

### Decode Mint

Decodes a mint account's metadata into a JSON file. It accepts either a single account or a list of accounts.

#### Usage

```bash
metaboss decode mint --account <MINT_ACCOUNT> -o <OUPUT_DIRECTORY>
```

The command will write the metadata JSON file to the output directory with the mint account as the name: e.g. `CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp.json`. The output option defaults to the current directory.

```bash
metaboss decode mint --list-file <LIST_FILE> -o <OUPUT_DIRECTORY>
```

The JSON list file should be an array of mint accounts to be decoded:

```json
["xSy...", "Cnb..." ...]
```

The command will write each metadata JSON file to the output directory as a separate file with the mint account as the name: e.g. `CQNKXw1rw2eWwi812Exk4cKUjKuomZ2156STGRyXd2Mp.json`. The output option defaults to the current directory.

As of v0.4.0, the default output will only be the `Data` struct matching the input format of the `update data` and `update data-all` commands. To get the full `Metadata` struct, use the `--full` option.

Use `--raw` to get the account data as raw bytes for debugging purposes.