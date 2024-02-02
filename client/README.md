# `hwlib` library and `hwctl` CLI tool

The client contains of two modules:

* [`hwlib`](./hwlib): Rust library for communicating with the API server
* [`hwctl`](./hwctl): CLI tool (written in Rust) that provides a user with the CLI tool for the `hwlib`


## Build the library (`hwlib`)

For now, the library contains the function to return a sample certification status. It depends on the environment variable `CERTIFICATION_STATUS` and accepts the following values:

* `0`: The system has not been seen (default behaviour even if the env variable is not defined).
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


## Use Python bindings

The `hwctl` lib can be used in Python code as well. We're using [pyo3](https://github.com/PyO3/pyo3) lib for creating Python bindings, so to use them, you need to have [maturin](https://github.com/PyO3/maturin) on your system installed. It requires virtual environment to be configured to work with it:

```bash
$ virtualenv venv
$ source venv/bin/activate
$ pip install maturin
```

Then, run the following command:

```bash
$ cd hwlib
$ maturin develop
```

Now you can use the lib in your Python code:

```python
>>> import hwlib
>>> hwlib.get_certification_status("https://example.com")
'{"NotSeen":{"status":"Not Seen"}}'
>>> import os
>>> os.environ["CERTIFICATION_STATUS"] = "2"
>>> hwlib.get_certification_status("https://example.com")
{'Certified': {'status': 'Certified', 'os': {'distributor': 'Ubuntu', 'description': 'Ubuntu 20.04.1 LTS', 'version': '20.04', 'codename': 'focal', 'kernel': {'name': 'Linux', 'version': '5.4.0-42-generic', 'signature': 'Sample Signature'}, 'loaded_modules': ['module1', 'module2']}, 'bios': {'firmware_revision': '1.0', 'release_date': '2020-01-01', 'revision': 'rev1', 'vendor': 'BIOSVendor', 'version': 'v1.0'}}}
```
