#!/bin/bash

cd server/

# Generate HTML schema
npx @redocly/cli build-docs ./schemas/openapi.yaml -o ./schemas/openapi.html
