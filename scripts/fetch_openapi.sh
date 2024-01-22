#!/bin/bash

cd server

# Start FastAPI app in the background
poetry run uvicorn hwapi.main:app --port 8002 &

# Wait for the server to start
sleep 5

# Fetch OpenAPI YAML
curl http://localhost:8002/openapi.yaml -o openapi.yaml

# Kill the FastAPI server
kill $!
