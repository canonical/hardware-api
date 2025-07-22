# Contributing to the Hardware API Client

This document provides the information needed to contribute to the Hardware API
Client.

For basic usage of the `hardware-api` client, refer to the client
[readme](./README.md).

For general contribution guidelines for the `hardware-api` project,
refer to the [contribution guide](../CONTRIBUTING.md).

## Snap Package

`hwctl` is packaged as a [snap]. To build the snap package locally, install
[`snapcraft`][snapcraft] and run the following command:

```shell
snapcraft pack
```

The above command will create a file in the form of
`hwctl_<version>_<architecture>.snap`. To install the snap locally, run the
following command:

```shell
sudo snap install --dangerous hwctl_<version>_<architecture>.snap
# The following interfaces are required to run the snap package
# They are not automatically connected when installing a local package
sudo snap connect hwctl:hardware-observe
sudo snap connect hwctl:kenrel-module-observe
sudo snap connect hwctl:system-observe
```

## Use Python Bindings

The `hwlib` library can be used in Python code as well. We're using the
[`pyo3`][pyo3] library to create Python bindings; to build them, you need to
have [`maturin`][maturin] installed on your system. It requires a virtual
environment to be configured to work with it:

```shell
virtualenv venv
source venv/bin/activate
pip install maturin
```

Then run the following commands:

```shell
cd hwlib
maturin develop
```

Now you can use the library in your Python code. The library requires root
access since we collect the hardware information using SMBIOS data. If you're
running it on a device that doesn't have SMBIOS data available, root privileges
are not required.

```python
$ sudo /path/to/venv/bin/python3  # or `python3`
>>> import hwlib
>>> hwlib.get_certification_status("https://hw.ubuntu.com")
```

## Test

Since we're using Python bindings, this library contains tests for both Rust
and Python code. To execute them, run the following commands in the `hwlib/`
directory:

- Run Rust tests: `$ cargo test`
- Run Python tests: You need to have [`tox`][tox] (`pip install tox`). Then you
  can run Python tests with `$ tox`


[snap]: https://snapcraft.io/hwctl
[snapcraft]: https://github.com/canonical/snapcraft
[pyo3]: https://github.com/PyO3/pyo3
[maturin]: https://github.com/PyO3/maturin
[tox]: https://github.com/tox-dev/tox
