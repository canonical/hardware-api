#!/bin/sh
set -xeu

python3 debian/tests/mock_server.py &
SERVER_PID=$!
sleep 1  # Give server time to start

export HW_API_URL="http://localhost:8089"
OUTPUT=$(hwctl)

EXPECTED_OUTPUT='{ "status": "Not Seen" }'

# Normalize both JSONs using jq for accurate comparison
if ! echo "$OUTPUT" | jq -S . > /tmp/out.json 2>/dev/null; then
    echo "Error: hwctl output is not valid JSON: $OUTPUT"
    kill $SERVER_PID
    exit 1
fi

if ! echo "$EXPECTED_OUTPUT" | jq -S . > /tmp/expected.json; then
    echo "Error: expected output is not valid JSON: $EXPECTED_OUTPUT"
    kill $SERVER_PID
    exit 1
fi

if ! diff -u /tmp/expected.json /tmp/out.json; then
    echo "Error: output does not match expected"
    kill $SERVER_PID
    exit 1
fi

kill $SERVER_PID

