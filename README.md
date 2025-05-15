# Hardware Information API (hwapi)

The repo contains the API server and client for retrieving hardware
information.

## Prerequisites for deploying locally

- Install [docker](https://docs.docker.com/engine/install/ubuntu/) and
  [setup
  permissions](https://docs.docker.com/engine/install/linux-postinstall/)
  for server deployment
- Install `rust` and `cargo` for client deployment
  (https://www.rust-lang.org/tools/install)

Also, to run the server locally, make sure that another application
doesn't use the port `8080`.

## Running API development server instance with Docker

Go to the `server/` directory in the project. Since the server needs
the data from the DB to work with, there are several options regarding
how you can stand up the environment.

### Using pre-populated database

You can use this option if you already have the SQLite DB file and
want to use it. Make sure that the DB is located under the `./server/`
directory.

You can stand up the environment by running the following commands:

```bash
# Assuming that the path to the DB file is ./hwapi.db
docker compose build --build-arg IMPORT_TOOL_PATH="" --build-arg DB_URL=sqlite:///./hwapi.db hwapi-dev
docker compose up --attach-dependencies hwapi-dev
```

### Seed the database from the script

This approach doesn't require internet access and populates your DB
with dummy data using the `scripts/seed_db.py` script.

```bash
export IMPORT_TOOL_PATH=./scripts/seed_db.py
docker compose up --attach-dependencies --build hwapi-dev
```

### Load the data from C3

This approach populates the DB with the data from C3 (staging instance
by default).  Keep in mind that importing data from staging or
production takes some time, you probably want to consider importing
data from your local C3 instance with sample data.

To build and run the container with staging data, execute the
following command:

```bash
docker compose up --attach-dependencies --build hwapi-dev
```

Alternatively, you can specify another C3 host (like production or the
local one) by specifying the `C3_URL`:

```bash
export C3_URL=http://your.c3.instance  # e.g https://certification.canonical.com
docker compose up --attach-dependencies --build hwapi-dev
```

## Accessing API schema

You can retrieve API schema in HTML, YAML, and JSON formats:

- The current version of the OpenAPI schema is browseable at
  [canonical.github.io/hardware-api](https://canonical.github.io/hardware-api).
- To access the HTML view for the API schema, just run the server and
  follow the [/#docs](http://127.0.0.1:8080/#docs) endpoint.
- Retrieve the schema in YAML from the running service by following
  the [/openapi.yaml](http://127.0.0.1:8080/v1/openapi.yaml) endpoint
- A copy of the [openapi.yaml](./server/schemas/openapi.yaml) is
  included in the repo, and it is enforced by a CI automation to be up
  to date.
- For getting its JSON version, follow the
  [/openapi.json](http://127.0.0.1:8080/openapi.json) endpoint.

## Build the library (`hwlib`)

Currently, the library contains the functionality to collect the
information about the machine hardware and OS, send the request to the
server, and get the machine certification status.

```bash
sudo apt-get install -y pkgconf libssl-dev  # Install the required debian dependencies
cd client/hwlib
cargo build
```

## Build and run the reference CLI tool (`hwctl`)

To check the machine certification status, run the following
command. It sends the request to the server URL defined by
`HW_API_URL` environment value (https://hw.ubuntu.com by default). The
library requires root access since we collect the hardware information
using SMBIOS data. If you're running it on a device that doesn't have
SMBIOS data available, root privileges are not required.

```bash
cd client/
cargo build
sudo ./target/debug/hwctl
```

To send the request to a different server, run the tool specifying
`HW_API_URL` environment variable:

```bash
cargo build
sudo HW_API_URL=https://your.server.url ./target/debug/hwctl
```
