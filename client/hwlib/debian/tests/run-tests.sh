#!/bin/sh
set -xeu

# Create cargo config directory
CARGO_TMP_DIR=$(mktemp -d)

# Create config.toml
cat > $CARGO_TMP_DIR/config.toml << EOL
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$(pwd)/rust-vendor"
EOL

# Run the actual test
CARGO_HOME=$CARGO_TMP_DIR cargo test --offline --manifest-path Cargo.toml --all-targets --all-features

rm -rf $CARGO_TMP_DIR
