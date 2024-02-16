# `hwlib` library and `hwctl` CLI tool

The client contains of two modules:

* [`hwlib`](./hwlib): Rust library for communicating with the API server
* [`hwctl`](./hwctl): CLI tool (written in Rust) that provides a user with the CLI tool for the `hwlib`


## Use Python Bindings

The `hwlib` lib can be used in Python code as well. We're using [pyo3](https://github.com/PyO3/pyo3) lib for creating Python bindings, so to build them, you need to have [maturin](https://github.com/PyO3/maturin) on your system installed. It requires virtual environment to be configured to work with it:

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
{'NotSeen':{'status':'Not Seen'}}
>>> import os
>>> os.environ["CERTIFICATION_STATUS"] = "2"
>>> hwlib.get_certification_status("https://example.com")
{'Certified': {'status': 'Certified', 'os': {'distributor': 'Ubuntu', 'description': 'Ubuntu 20.04.1 LTS', 'version': '20.04', 'codename': 'focal', 'kernel': {'name': 'Linux', 'version': '5.4.0-42-generic', 'signature': 'Sample Signature'}, 'loaded_modules': ['module1', 'module2']}, 'bios': {'firmware_revision': '1.0', 'release_date': '2020-01-01', 'revision': 'rev1', 'vendor': 'BIOSVendor', 'version': 'v1.0'}}}
```


## Run Tests

Since we're using python bindings, this library contains tests for both Rust and Python code. To execute them, run the following commands in the `hwlib/` directory:

* Run Rust tests: `$ cargo test -- --test-threads=1`
* For Python tests, you need to have `tox` on your system installed: `pip install tox`. Then, you can run Python tests with tox `$ tox`


## Build Deb Package

This section describes how to pack `hwlib` as a debian package. First, make sure that you have `devscripts` and `dput` installed on your system.

Before creating an archive, make sure you don't have any files and directories like `target/` in the `client/hwlib` directory:

```bash
$ cd client/hwlib/
:client/hwlib/$ git clean -dffx
```

Then we need to create the `.whl` file for the `hwlib` locally, since `maturin` python library is not available as a debian package and we cannot include this step to the [rules](./debian/rules) file. To do it, run the following commands in the pre-created virtual environment. And since we're building for the mantic release, it needs to be run on Ubuntu 23.10.

```bash
(venv) :client/hwlib$ pip install maturin
(venv) :client/hwlib$ maturin build --release -b pyo3 -i /path/to/venv/bin/python3
```

After that, you may probably need to update the package version. Make sure that the version is unique, otherwise it'll be rejected. To do this, run the following commands:

```bash
# Version string should have the format X.Y.Z and optionally you can
# specify "~devN" suffix. Examples: 1.0.0, 1.2.3~dev1
:client/hwlib$ export VERSION=X.Y.Z[~devN]
:client/hwlib$ python3 ../../scripts/update_package_version.py $VERSION <your.email>@canonical.com
:client/hwlib$ cargo update
```

After that, create the archive and publish the package:

```bash
:client/hwlib$ tar czvf ../hwlib_$VERSION.orig.tar.gz --exclude debian .
:client/hwlib$ debuild -S -sa -k<your_gpg_key_short_ID>
:client/hwlib$ dput ppa:<ppa_name> ../hwlib_$VERSION.orig.tar.gz
```
