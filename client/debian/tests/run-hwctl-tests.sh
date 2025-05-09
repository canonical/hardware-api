#!/bin/sh
set -xeu

systemd-run --unit=mock-hw-server \
            --property=NotifyAccess=all \
            --property=Type=notify \
            --slice=background.slice \
            --quiet \
            python3 "$(realpath "$(dirname "$0")")/mock_server.py"

export HW_API_URL="http://localhost:8089"
OUTPUT=$(hwctl)

EXPECTED_OUTPUT='{ "status": "Not Seen" }'

# Normalize both JSONs using jq for accurate comparison
if ! echo "$OUTPUT" | jq -S . > /tmp/out.json 2>/dev/null; then
    echo "Error: hwctl output is not valid JSON: $OUTPUT"
    systemctl stop mock-hw-server
    exit 1
fi

if ! echo "$EXPECTED_OUTPUT" | jq -S . > /tmp/expected.json; then
    echo "Error: expected output is not valid JSON: $EXPECTED_OUTPUT"
    systemctl stop mock-hw-server
    exit 1
fi

if ! diff -u /tmp/expected.json /tmp/out.json; then
    echo "Error: output does not match expected"
    systemctl stop mock-hw-server
    exit 1
fi

systemctl stop mock-hw-server
echo "Integration test passed successfully"
