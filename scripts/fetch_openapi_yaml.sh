#!/bin/bash

cd server

# Start FastAPI app in the background
poetry run uvicorn hwapi.main:app --port 8002 &

# Wait for the server to start
sleep 5

# Fetch OpenAPI YAML
curl http://localhost:8002/v1/openapi.yaml -o schemas/openapi.yaml

# Kill the FastAPI server
kill $!

# Add schema to the committed files
git add openapi.yaml