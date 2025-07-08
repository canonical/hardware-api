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

# Delete .sh, .a, and .o files in the vendored directory. Currently it
# cannot be done by using the cargo-vendor-filterer tool itself due to
# the lack of the glob patterns in exclude paths support
# <https://github.com/coreos/cargo-vendor-filterer/issues/122>. For
# each package directory, remove binary files and update its checksum
for package_dir in "$CARGO_VENDOR_DIR"/*; do
    if [ -d "$package_dir" ]; then
        if [ -n "$(find "$package_dir" -name "*.sh" -o -name "*.a" -o -name "*.o")" ]; then

            find "$package_dir" -name "*.sh" -exec rm -f {} +
            find "$package_dir" -name "*.a" -exec rm -f {} +
            find "$package_dir" -name "*.o" -exec rm -f {} +

            checksum_file="$package_dir/.cargo-checksum.json"
            tmp_file=$(mktemp)
            jq '.files |= with_entries(select(.key | (endswith(".sh") or endswith(".a") or endswith(".o")) | not))' "$checksum_file" > "$tmp_file"
            mv "$tmp_file" "$checksum_file"
        fi
    fi
done

echo "Filtered vendored dependencies and updated checksums."
