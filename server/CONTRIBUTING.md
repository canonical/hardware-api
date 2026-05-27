# Contributing to the Hardware API Server

This document provides the information needed to contribute to the Hardware API
Server.

For general contribution guidelines for the `hardware-api` project,
refer to the [contribution guide](../CONTRIBUTING.md).

## Development Environment

You can create an environment for development with [`uv`][uv]:

```shell
uv sync
source .venv/bin/activate
```

## Testing

This project uses [`tox`][tox] for managing test environments. There are some
pre-configured environments that can be used for linting and formatting code
when you're preparing contributions to the project:

```shell
tox run -e format       # update your code according to linting rules
tox run -e lint         # code style
tox run -e unit         # unit (coverage) tests
tox run -e schema       # update the openapi schema
tox run -e check-schema # check the openapi schema for updates
tox                     # runs 'format', 'lint', 'unit', and 'schema'
```

## Deploy the Server

### Deploy with Docker

The dev stack runs the server against a PostgreSQL container defined in
[`docker-compose.yml`](./docker-compose.yml). Stand it up with:

```shell
docker compose up --attach-dependencies --build hwapi-dev
```

The server starts against an empty schema. Populate the database by running
the one-shot `hwapi-update` service, which executes the script pointed at by
the `IMPORT_TOOL_PATH` environment variable.

#### Populating the Database

The `hwapi-update` service lives behind the `tools` profile and is invoked
on-demand against the running `db` container. Set `IMPORT_TOOL_PATH` to choose
which importer to run:

| `IMPORT_TOOL_PATH`               | What it does                                                 |
| -------------------------------- | ------------------------------------------------------------ |
| `scripts/update_db.py` (default) | Wait for the DB, ensure schema, pull fresh data from C3      |
| `scripts/import_from_c3.py`      | Ensure schema, pull data from C3                             |
| `scripts/seed_db.py`             | Populate the DB with dummy data — no network required        |
| `scripts/import_test_data.py`    | Import with mocked C3 responses from `scripts/c3_test_data/` |

Run with the default (refresh from C3):

```shell
docker compose run --rm hwapi-update
```

Run a specific importer by overriding `IMPORT_TOOL_PATH`:

```shell
IMPORT_TOOL_PATH=scripts/seed_db.py docker compose run --rm hwapi-update
```

#### Pointing at a Different C3 Instance

Importing from staging or production C3 takes some time; you may prefer your
own C3 instance with sample data. Set `C3_URL` before invoking `hwapi-update`:

```shell
export C3_URL=http://your.c3.instance  # e.g., https://certification.canonical.com
docker compose run --rm hwapi-update
```

#### Pointing at a Different Database

By default the dev stack uses the bundled `db` service. To run the server or
the importer against an existing database, set `DB_URL` to any SQLAlchemy URL
(PostgreSQL is supported out of the box via `psycopg`):

```shell
export DB_URL=postgresql+psycopg://user:password@db.example.com:5432/hwapi
docker compose run --rm hwapi-update
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

[uv]: https://docs.astral.sh/uv
[tox]: https://tox.wiki/en/latest/index.html
[canonical.github.io/hardware-api]: https://canonical.github.io/hardware-api
