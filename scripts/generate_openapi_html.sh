#!/bin/bash

cd server/schemas/

# Generate HTML schema
npx @redocly/cli build-docs openapi.yaml -o openapi.html

git add openapi.html
