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
>>> import asyncio
>>> async def check_certification_status(url):
...     result = await hwlib.get_certification_status(url)
...     print(result)
...
>>> asyncio.run(check_certification_status("https://example.com"))
{'NotSeen':{'status':'Not Seen'}}
>>> import os
>>> os.environ["CERTIFICATION_STATUS"] = "2"
>>> asyncio.run(check_certification_status("https://example.com"))
{'Certified': {'status': 'Certified', 'os': {'distributor': 'Ubuntu', 'description': 'Ubuntu 20.04.1 LTS', 'version': '20.04', 'codename': 'focal', 'kernel': {'name': 'Linux', 'version': '5.4.0-42-generic', 'signature': 'Sample Signature'}, 'loaded_modules': ['module1', 'module2']}, 'bios': {'firmware_revision': '1.0', 'release_date': '2020-01-01', 'revision': 'rev1', 'vendor': 'BIOSVendor', 'version': 'v1.0'}}}
```


## Run Tests

Since we're using python bindings, this library contains tests for both Rust and Python code. To execute them, run the following commands in the `hwlib/` directory:

* Run Rust tests: `$ cargo test`.
* For Python tests, you need to have `tox` on your system installed: `pip install tox`. Then, you can run Python tests with tox `$ tox`.
