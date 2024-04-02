#!/bin/bash

# Don't populate existing DB with dummy data
if ! [ -f data/hwapi.db ]; then
    mkdir -p data;
    python scripts/seed_db.py;
fi

uvicorn hwapi.main:app --host 0.0.0.0 --port 8080 --reload
