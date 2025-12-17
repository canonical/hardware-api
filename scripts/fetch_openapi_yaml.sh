#!/bin/bash

# Run only if server code has been modified
if git diff --cached --name-only | grep --quiet "server"
then
    cd server
    # Start FastAPI app in the background
    uv run uvicorn hwapi.main:app --port 8002 &

    # Wait for the server to start
    sleep 5

    # Fetch OpenAPI YAML
    curl http://localhost:8002/v1/openapi.yaml -o schemas/openapi.yaml

    # Kill the FastAPI server
    kill $!

    # Add schema to the committed files
    git add schemas/openapi.yaml
fi
