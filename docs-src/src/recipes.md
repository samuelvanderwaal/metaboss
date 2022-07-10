## Update List of NFT Metadata

1. Get your NFT mint list using `metaboss snapshot` or another tool.
2. Decode all the metadata into files using `metaboss decode mint -L <NFT_MINT_LIST_FILE> --full -o <DATA_FILES_DIR>`.
3. Update the specific data you want changed in each file in the `<DATA_FILES_DIR>`.
4. Update the NFTs with `metaboss update data-all -d <DATA_FILES_DIR>`.
