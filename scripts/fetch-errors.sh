RED() { echo $'\e[1;31m'$1$'\e[0m'; }
GRN() { echo $'\e[1;32m'$1$'\e[0m'; }
CYN() { echo $'\e[1;36m'$1$'\e[0m'; }

abort_on_error() {
    if [ ! $1 -eq 0 ]; then
        RED "Aborting: operation failed"
        exit 1
    fi
}

download_file() {
  curl -L $1 --output "$2"
  abort_on_error $?

  SIZE=$(wc -c "$2" | grep -oE "[0-9]+" | head -n 1)

  if [ $SIZE -eq 0 ]; then
      RED "Aborting: could not download Sugar distribution"
      exit 1
  fi
}

DOWNLOAD_DIST="$PWD/src/error_files"
WTF_ERROR_DIST="$PWD/src/wtf_errors.rs"
mkdir $DOWNLOAD_DIST
rm -rf "$WTF_ERROR_DIST"
touch "$WTF_ERROR_DIST"


CYN  "🍬 Metaboss Error Fetching script"
echo "---------------------------------------"
echo ""

echo "$(CYN "1.") 🖥  $(CYN "Downlading error files")"
echo ""

download_file "https://raw.githubusercontent.com/project-serum/anchor/master/lang/src/error.rs" "$DOWNLOAD_DIST/anchor-error.rs"

download_file "https://raw.githubusercontent.com/metaplex-foundation/metaplex-program-library/master/candy-machine/program/src/errors.rs" "$DOWNLOAD_DIST/candy-error.rs"

download_file "https://raw.githubusercontent.com/metaplex-foundation/metaplex-program-library/master/auction-house/program/src/errors.rs" "$DOWNLOAD_DIST/auction-house-error.rs"

download_file "https://raw.githubusercontent.com/metaplex-foundation/metaplex-program-library/master/auctioneer/program/src/errors.rs" "$DOWNLOAD_DIST/auctioneer-error.rs"

download_file "https://raw.githubusercontent.com/metaplex-foundation/metaplex-program-library/master/token-metadata/program/src/error.rs" "$DOWNLOAD_DIST/metadata-error.rs"

echo ""
echo "$(CYN "2.") 📤  $(CYN "Parsing errors")"
echo ""
metaboss parse-errors file -l error

cargo fmt --all

rm -Rf "$DOWNLOAD_DIST"



