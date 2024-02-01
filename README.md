# Hardware Information API (hwapi)

The repo contains the API server and client for retrieving hardware information.

## Build and run the server

```bash
poetry install
poetry run uvicorn hwapi.main:app --reload
```

## View OpenAPI schema for the server

A HTML view of the OpenAPI schema matching the `main` branch is viewable at
[canonical.github.io/hardware-api](https://canonical.github.io/hardware-api)

## Build the library (`hwlib`)

For now, the library contains the function to return a sample certification status. It depends on the environment variable `CERTIFICATION_STATUS` and accepts the following values:

* `0`: The system has not been seen.
* `1`: The system is partially certified (we haven't seen this specific system, but some of its hardware components have been tested on other systems).
* `2`: This system has been certified (but probably for other Ubuntu release).

```bash
$ cd client/hwlib
$ cargo build
```

## Build and run the reference CLI tool (`hwctl`)

```bash
$ export CERTIFICATION_STATUS=2
$ cd client/hwctl
$ cargo run
```

This is the output you should get running the commands above:

```bash
   Compiling hwlib v0.1.0 (/home/nadzeya/Work/Canonical/hiapi/client/hwlib)
   Compiling hwctl v0.1.0 (/home/nadzeya/Work/Canonical/hiapi/client/hwctl)
    Finished dev [unoptimized + debuginfo] target(s) in 2.00s
     Running `/home/nadzeya/Work/Canonical/hiapi/target/debug/hwctl`
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
