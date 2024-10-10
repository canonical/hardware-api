#!/bin/bash
# Reference: https://github.com/ubuntu/authd/blob/86d9d82/debian/vendor-rust.sh
set -eu

CARGO_VENDOR_DIR="${CARGO_VENDOR_DIR:-rust-vendor}"

# We need a filtered vendored directory
if [ ! $(which cargo-vendor-filterer) ]; then
    echo "ERROR: could not find cargo-vendor-filterer in PATH to filter vendored dependencies." >&2
    echo "Please install cargo-vendor-filterer to run this script. More info at https://github.com/coreos/cargo-vendor-filterer." >&2
    exit 3
fi

test -n "$(command -v jq)" || (echo "jq is required to run this script. Try installing it with 'sudo apt install jq'" && exit 1);


cargo vendor-filterer "$CARGO_VENDOR_DIR"

# Delete .sh and .a files in the vendor/vcpkg directory if it exists
if [ -d "$CARGO_VENDOR_DIR/vcpkg" ]; then
    find "$CARGO_VENDOR_DIR/vcpkg" -name "*.sh" -exec rm -f {} +
    find "$CARGO_VENDOR_DIR/vcpkg" -name "*.a" -exec rm -f {} +
    # Remove checksums for the deleted *.sh and *.a files that are not used by the code but generate lintian errors
    for json_file in $(find "${CARGO_VENDOR_DIR}/vcpkg" -name ".cargo-checksum.json"); do
        tmp_file=$(mktemp)
        jq '.files |= with_entries(select(.key | (endswith(".sh") or endswith(".a")) | not))' "$json_file" > "$tmp_file"
        mv "$tmp_file" "$json_file"
    done
fi

echo "Filtered vendored dependencies and updated checksums."
