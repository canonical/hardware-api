#!/bin/bash

file_path=$(echo $DB_URL | sed 's/sqlite:\/\/\///')

# Don't populate existing DB with dummy data
if ! [ -f "$file_path" ]; then
    echo "Populating the DB with sample data"
    mkdir -p data;
    python scripts/seed_db.py;
fi

uvicorn hwapi.main:app --host 0.0.0.0 --port 8080 --reload
