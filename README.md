
# Hardware Information API (hwapi)

The repo contains the API server and client for retrieving hardware information.

## Prerequisites for deploying locally

- Install [docker](https://docs.docker.com/engine/install/ubuntu/) and
  [setup permissions](https://docs.docker.com/engine/install/linux-postinstall/)
  for server deployment
- Install `rust` and `cargo` for client deployment
  (https://www.rust-lang.org/tools/install)

Also, to run the server locally, make sure that another application doesn't use the port `8080`.

## Running API development server instance with Docker

Go to the `server/` directory in the project. Since the server needs the data from the DB to work with,
there are several options regarding how you can stand up the environment.

### Using pre-populated database

You can use this option if you already have the SQLite DB file and want to use it. Make sure that the DB
is located under the `./server/` directory.

You can stand up the environment by running the following commands:

```bash
# Assuming that the path to the DB file path is ./hwapi.db
docker-compose build --build-arg IMPORT_TOOL_PATH="" --build-arg DB_URL=sqlite:///./hwapi.db hwapi-dev
docker-compose up --attach-dependencies hwapi-dev
```

### Seed the database from the script

This approach doesn't require internet access and populates your DB with dummy data using the
`scripts/seed_db.py` script.

```bash
export IMPORT_TOOL_PATH=./scripts/seed_db.py
docker-compose up --attach-dependencies --build hwapi-dev
```

To verify that it works, you can make the following request from the host
(you should receive the Certified response):

```bash
curl http://0.0.0.0:8080/v1/certification/status -X POST -H "Content-Type: application/json" \
-d '{"vendor": "Dell", "model": "ChengMing 3980"}' -s | python3 -m json.tool
```

### Load the data from C3

This approach populates the DB with the data from C3 (staging instance by default). To build and run the
container, execute the following command:

```bash
docker-compose up --attach-dependencies --build hwapi-dev
```

To verify that it works, make the following request from the host (you should receive the Certified response):

```bash
curl http://0.0.0.0:8080/v1/certification/status -X POST -H "Content-Type: application/json" \
-d '{"vendor": "HP", "model": "Z8 G4 Workstation"}' -s | python3 -m json.tool
```

Alternatively, you can specify another C3 host (like production or the local one) by specifying the `C3_URL`:

```bash
export C3_URL=http://your.c3.instance  # e.g https://certification.canonical.com
docker-compose up --attach-dependencies --build hwapi-dev
```

## Accessing API schema

You can retrieve API schema in HTML, YAML, and JSON formats:

- The current version of the OpenAPI schema is browseable at
  [canonical.github.io/hardware-api](https://canonical.github.io/hardware-api).
- To access the HTML view for the API schema, just run the server and follow the
  [/#docs](http://127.0.0.1:8080/#docs) endpoint.
- A self-contained HTML representation of the schema is also included in the
  repository: [openapi.html](./server/schemas/openapi.html).
- Retrieve the schema in YAML from the running service by following the
  [/openapi.yaml](http://127.0.0.1:8080/v1/openapi.yaml) endpoint
- A copy of the [openapi.yaml](./server/schemas/openapi.yaml) is included in the
  repo, and it is enforced by a CI automation to be up to date.
- For getting its JSON version, follow the
  [/openapi.json](http://127.0.0.1:8080/openapi.json) endpoint.

## Build the library (`hwlib`)

For now, the library contains the function to return a sample certification
status. It depends on the environment variable `CERTIFICATION_STATUS` and
accepts the following values:

- `0`: The system has not been seen (default behaviour even if the env variable
  is not defined).
- `1`: The system is partially certified (we haven't seen this specific system,
  but some of its hardware components have been tested on other systems).
- `2`: This system has been certified (but probably for other Ubuntu release).

```bash
cd client/hwlib
cargo build
```

## Build and run the reference CLI tool (`hwctl`)

```bash
export CERTIFICATION_STATUS=2
cd client/hwctl
cargo run
```

This is the output you should get running the commands above:

```bash
    Finished dev [unoptimized + debuginfo] target(s) in 2.00s
Certified(
    CertifiedResponse {
        status: "Certified",
        os: OS {
            distributor: "Ubuntu",
            description: "Ubuntu 20.04.1 LTS",
            version: "20.04",
            codename: "focal",
            kernel: KernelPackage {
                name: "Linux",
                version: "5.4.0-42-generic",
                signature: "Sample Signature",
            },
            loaded_modules: [
                "module1",
                "module2",
            ],
        },
        bios: Bios {
            firmware_revision: "1.0",
            release_date: "2020-01-01",
            revision: "rev1",
            vendor: "BIOSVendor",
            version: "v1.0",
        },
    },
)
```

## Building `hwctl` snap

To build and install `hwctl` as a snap locally, do the following after
[installing snapcraft and a build provider for it](https://snapcraft.io/docs/snapcraft-setup):

```bash
snapcraft --bind-ssh  # --verbose
sudo snap install ./hwctl_[version].snap --dangerous
```
