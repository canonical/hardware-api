# Hardware Information API (hwapi)

The repo contains the API server and client for retrieving hardware information.

## Prerequisites for Deploying Locally

- Install [docker](https://docs.docker.com/engine/install/ubuntu/) and
  [setup permissions](https://docs.docker.com/engine/install/linux-postinstall/)
  for server deployment
- Install `rust` and `cargo` for client deployment
  (https://www.rust-lang.org/tools/install)

## Running API Development Server Instance With Docker

Go to the `server/` directory in the project. The following command builds the
hwapi server, and during the build time it imports the data from C3 staging server:

```bash
docker-compose up --attach-dependencies --build hwapi-dev
```

You can also customise the build arguments by specifying the following environment values:

* `SCRIPT_NAME` (default `import_from_c3.py`): Specify the script from the
  `server/scripts/` directory to use for populating the DB. If you don't want to import
  the data from C3, you can use `seed_db.py` script, which doesn't require access to C3
* `C3_URL` (default `https://certification.staging.canonical.com`): If you want to
  use production data or the data from your local C3 instance, you can change this variable

After standing up the container, you should be able to access the server via 
this URL: http://127.0.0.1:8080

## Accessing API Schema

You can retrieve API schema in HTML, YAML, and JSON formats:

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

## Build The Library (`hwlib`)

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

## Build and Run the Reference CLI Tool (`hwctl`)

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

## Building `hwctl` Snap

To build and install `hwctl` as a snap locally, do the following after
[installing snapcraft and a build provider for it](https://snapcraft.io/docs/snapcraft-setup):

```bash
snapcraft --bind-ssh # --verbose
sudo snap install ./hwctl_[version].snap --dangerous
```
