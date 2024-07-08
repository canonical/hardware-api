#!/bin/bash
set -eu

CARGO_VENDOR_DIR="${CARGO_VENDOR_DIR:-vendor}"

# We need a filtered vendored directory
if [ ! $(which cargo-vendor-filterer) ]; then
    echo "ERROR: could not find cargo-vendor-filterer in PATH to filter vendored dependencies." >&2
    echo "Please install cargo-vendor-filterer to run this script. More info at https://github.com/coreos/cargo-vendor-filterer." >&2
    exit 3
fi

cargo vendor-filterer "$CARGO_VENDOR_DIR"

# Some crates are shipped with .a files, which get removed by the helpers during the package build as a safety measure.
# This results in cargo failing to compile, since the files (which are listed in the checksums) are not there anymore.
# For those crates, we need to remove their checksums
for json_file in $(find "$CARGO_VENDOR_DIR" -name ".cargo-checksum.json"); do
    tmp_file=$(mktemp)
    jq '.files |= with_entries(select(.key | endswith(".a") | not))' "$json_file" > "$tmp_file"
    mv "$tmp_file" "$json_file"
done

echo "Filtered vendored dependencies and updated checksums."
