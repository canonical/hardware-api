#!/bin/bash

# Create cargo config directory
mkdir -p /tmp/.cargo

# Create config.toml
cat > /tmp/.cargo/config.toml << EOL
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/tmp/vendor"
EOL

# Run the actual test
ls -la
pwd
CARGO_HOME=/tmp cargo test --offline \
    --manifest-path ../../Cargo.toml \
    --all-targets --all-features
