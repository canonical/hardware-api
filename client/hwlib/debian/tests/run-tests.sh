#!/bin/bash

# Create cargo config directory
mkdir -p /tmp/.cargo

# Create config.toml
cat > /tmp/.cargo/config.toml << EOL
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "$(pwd)/rust-vendor"
EOL

# Run the actual test
CARGO_HOME=/tmp/.cargo/ cargo test --offline --manifest-path Cargo.toml --all-targets --all-features
