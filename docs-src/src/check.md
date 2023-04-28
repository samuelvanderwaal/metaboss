## Check

### Metadata Values

Check for a specific metadata value in downloaded metadata files.

### Usage

Run 

```bash
metaboss decode mint -L <mint_list_json> -o <output_dir> --full
```

to decode all the metadata files in the `mint_list_json` file and save them to the `output_dir` directory.

Then run

```bash
metaboss check metadata-value -d <files_dir> METADATA_VALUE=VALUE
```

where the `files_dir` is the location of the metadata files downloaded in the previous command and `METADATA_VALUE` is one of the following
values:

- `name`
- `symbol`
- `uri`
- `sfbp`
- `creators`
- `update_authority`
- `primary_sale_happened`
- `is_mutable`
- `token_standard`
- `collection_parent`
- `collection_verified`
- `rule_set`

and `VALUE` is the specific value of the field you wish to check for in the metadata files.

E.g., to see if all the metadata files in the `output_dir` directory have a specific `update_authority` value:

```bash
metaboss check metadata-value -d my_collection_files/ update_authority="PanbgtcTiZ2PveV96t2FHSffiLHXXjMuhvoabUUKKm8"
```

The command will print a list of files that do not have the specified value as well as create a files with the list of mints.
The mint file will have the name format: "mb_check_mints_<METADATA_VALUE>.json" and will be created in the directory where the command is run.
In the above example the mint file will be "mb_check_mints_update_authority.json" and will contain all the mints that do not have the specified
update authority. This will allow you to easily rerun an update or set command to fix metadata values that weren't set properly.

The "name" field will check that the name on the metadata *contains* the name you specify so you can check for partial matches.
E.g. if your collection's name format is "MyCollection #xx" you can set the name to be "MyCollection" and it will match all the metadata files
that have the name "MyCollection" in their name.