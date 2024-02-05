# Hardware Information API (hwapi)

The repo contains the API server and client for retrieving hardware information.


## Prerequisites for Deploying Locally

* `poetry` vesion 1.6.1 or later for servier deployment (https://python-poetry.org/docs/)
* `rust` and `cargo` for client deployment (https://www.rust-lang.org/tools/install)


## Running API Server

Go to the `server/` directory in the project and run the following commands:

```bash
$ poetry install
$ poetry run uvicorn hwapi.main:app --reload
```

Then you can access the server via this URL: http://127.0.0.1:8000

For information regarding accessing the API schema, read [server README](./server/README.md)


## Accessing API schema

You can retrieve API schema in HTML, YAML, and JSON formats:

- To access the HTML view for the API schema, just run the server and follow the [/#docs](http://127.0.0.1:8000/#docs) endpoint.
- A self-contained HTML representation of the schema is also included in the repository: [openapi.html](./schemas/openapi.html).
- Retrieve the schema in YAML from the running service by following the [/openapi.yaml](http://127.0.0.1:8000/v1/openapi.yaml) endpoint
- A copy of the [openapi.yaml](./schemas/openapi.yaml) is included in the repo, and it is enforced by a CI automation to be up to date.
- For getting its JSON version, follow the [/openapi.json](http://127.0.0.1:8000/openapi.json) endpoint.


## Build the library (`hwlib`)

For now, the library contains the function to return a sample certification status. It depends on the environment variable `CERTIFICATION_STATUS` and accepts the following values:

* `0`: The system has not been seen (default behaviour even if the env variable is not defined).
* `1`: The system is partially certified (we haven't seen this specific system, but some of its hardware components have been tested on other systems).
* `2`: This system has been certified (but probably for other Ubuntu release).

```bash
$ cd client/hwlib
$ cargo build
```


## Build and Run the Reference CLI Tool (`hwctl`)

```bash
$ export CERTIFICATION_STATUS=2
$ cd client/hwctl
$ cargo run
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
