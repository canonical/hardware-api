#!/bin/bash

# Run only if server code has been modified
if git diff --cached --name-only | grep --quiet "server"
then
    cd server/schemas/

    which npx
    # Generate HTML schema
    npx @redocly/cli build-docs openapi.yaml -o openapi.html

    git add openapi.html
fi
