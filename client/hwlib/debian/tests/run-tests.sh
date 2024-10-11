#!/bin/sh
set -xeu

# Create cargo config directory
mkdir -p /tmp/.cargo-autopkgtest

# Create config.toml
cat > /tmp/.cargo-autopkgtest/config.toml << EOL
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$(pwd)/rust-vendor"
EOL

# Run the actual test
CARGO_HOME=/tmp/.cargo-autopkgtest/ cargo test --offline --manifest-path Cargo.toml --all-targets --all-features
