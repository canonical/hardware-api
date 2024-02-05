# `hwlib` library and `hwctl` CLI tool

The client contains of two modules:

* [`hwlib`](./hwlib): Rust library for communicating with the API server
* [`hwctl`](./hwctl): CLI tool (written in Rust) that provides a user with the CLI tool for the `hwlib`


## Use Python Bindings

The `hwlib` lib can be used in Python code as well. We're using [pyo3](https://github.com/PyO3/pyo3) lib for creating Python bindings, so to use them, you need to have [maturin](https://github.com/PyO3/maturin) on your system installed. It requires virtual environment to be configured to work with it:

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
