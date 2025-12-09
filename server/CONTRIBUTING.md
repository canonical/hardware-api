# Contributing to the Hardware API Server

This document provides the information needed to contribute to the Hardware API
Server.

For general contribution guidelines for the `hardware-api` project,
refer to the [contribution guide](../CONTRIBUTING.md).

## Development Environment

You can create an environment for development with [`poetry`][poetry]:

```shell
poetry install
```

## Testing

This project uses [`tox`][tox] for managing test environments. There are some
pre-configured environments that can be used for linting and formatting code
when you're preparing contributions to the project:

```shell
tox run -e format  # update your code according to linting rules
tox run -e lint    # code style
tox run -e unit    # unit (coverage) tests
tox                # runs 'format', 'lint', and 'unit'
```

## Deploy the Server

### Deploy with Docker

The server needs the data from the database to work with; there are several
options to stand up the environment.

#### Using a Pre-populated Database

You can use this option if you already have the SQLite DB file and
want to use it. Make sure that the DB is located under the `./server/`
directory.

You can stand up the environment by running the following commands:

```shell
# Assuming that the path to the DB file is ./hwapi.db
docker compose build --build-arg IMPORT_TOOL_PATH="" --build-arg DB_URL=sqlite:///./hwapi.db hwapi-dev
docker compose up --attach-dependencies hwapi-dev
```

#### Seed the Database from the Script

This approach doesn't require internet access and populates your DB
with dummy data using the [`scripts/seed_db.py`](./scripts/seed_db.py) script.

```shell
export IMPORT_TOOL_PATH=./scripts/seed_db.py
docker compose up --attach-dependencies --build hwapi-dev
```

#### Load the Data from C3

This approach populates the database with the data from C3 (staging instance
by default). Keep in mind that importing data from staging or
production takes some time, you probably want to consider importing
data from your local C3 instance with sample data.

To build and run the container with staging data, execute the
following command:

```shell
docker compose up --attach-dependencies --build hwapi-dev
```

Alternatively, you can specify another C3 host (e.g., production or local) by
specifying the `C3_URL` environment variable:

```bash
export C3_URL=http://your.c3.instance  # e.g., https://certification.canonical.com
docker compose up --attach-dependencies --build hwapi-dev
```

## Access the API Schema

You can retrieve the API schema in HTML, YAML, and JSON formats:

- The current version of the OpenAPI schema is available under
  [canonical.github.io/hardware-api].
- A copy of the [`openapi.yaml`](./schemas/openapi.yaml) is included in the
  repository, and it is enforced by a CI automation to be up to date.
- To access the HTML view for the API schema, run the server and access the
  `/#docs` endpoint.
- To access the YAML schema, run the server and access the `/openapi.yaml`
  endpoint.
- To access the JSON schema, run the server and access the `/openapi.json`
  endpoint.

[poetry]: https://python-poetry.org/docs/
[tox]: https://tox.wiki/en/latest/index.html
[canonical.github.io/hardware-api]: https://canonical.github.io/hardware-api
