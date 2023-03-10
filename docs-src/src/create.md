## Create

### Fungible

Create a new SPL token mint and corresponding metadata account.

#### Usage

Specify the token decimals and path to a metadata file that contains the `name`, `symbol` and `uri` fields in a JSON format.


```bash
metaboss create fungible -d <decimals> -m <metadata_file>
```

E.g.:

// crab.json
```json
{
  "name": "Crabbie",
  "symbol": "CRAB",
  "uri": "https://arweave.net/KZDlKw8aCG4kfZtj9Qmh8tmYpH4Q287P_jmUtkl2s-k"
}
```

```bash
metaboss create fungible -d 6 -m crab.json
```

Use the `--initial-suply` option to mint the specified amount directly to your keypair upon token creation. The amount is specified is the UI amount as a float. E.g. `--initial-supply 10.123` with three decimals will mint 10123 base unit tokens to your keypair.

```bash

```bash
metaboss create fungible -d 3 -m crab.json --initial-supply 10.1
```

### Metadata

Decorate an existing SPL token mint with metadata.

#### Usage

Specify the existing mint address and path to a metadata file that contains the `Data` sub-struct of the metadata struct in a JSON format.

E.g.:

// crab.json
```json
{
  "name": "Crabbie",
  "symbol": "CRAB",
  "uri": "https://arweave.net/KZDlKw8aCG4kfZtj9Qmh8tmYpH4Q287P_jmUtkl2s-k",
  "sellerFeeBasisPoints": 0,
  "creators": null
}
```

```bash
metaboss create metadata -a <mint_address> -m <metadata_file>
```