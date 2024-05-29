#!/bin/bash

# Run only if server code has been modified
if git diff --cached --name-only | grep --quiet "server"
then
    cd server/schemas/

    if ! command -v npx
    then
        echo "Install nodejs according to the instructions in server/README.md"
        exit 1
    fi
    # Generate HTML schema
    npx @redocly/cli build-docs openapi.yaml -o openapi.html

    git add openapi.html
fi
