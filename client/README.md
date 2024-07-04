# `hwlib` library and `hwctl` CLI tool

The client contains of two modules:

* [`hwlib`](./hwlib): Rust library for communicating with the API server
* [`hwctl`](./hwctl): CLI tool (written in Rust) that provides a user with the CLI tool for the `hwlib`

## Build package in offline mode with vendored dependencies

To build the package using the vendored dependencies and then run it in offline mode,
execute the following commands:

```bash
cd client/hwlib
# By default, the vendored dependencies are stored under the ./vendor/ directory
./debian/vendor-rust.sh
```

Then, modify your `~/.cargo/config.toml` (or `~/.cargo/config` if you use an older cargo
version) to use the vendored dependencies. Don't forget to specify the correct path:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/path/to/hardware-api/client/hwlib/vendor"
```

Currently, there is no way to change these settings per project, so cargo will use
these vendored dependencies for all your Rust projects. You can remove these lines
after finishing the work on the `hwlib`.

Now you can build the project in offline mode:

```bash
cd client/
cargo build --offline
```

## Use Python bindings

The `hwlib` lib can be used in Python code as well. We're using [pyo3](https://github.com/PyO3/pyo3)
lib for creating Python bindings, so to build them, you need to have [maturin](https://github.com/PyO3/maturin)
on your system installed. It requires virtual environment to be configured to work with it:

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
{'status':'Not Seen'}
>>> import os
>>> os.environ["CERTIFICATION_STATUS"] = "2"
>>> hwlib.get_certification_status("https://example.com")
{'bios': {'firmware_revision': '1.0', 'release_date': '2020-01-01', 'revision': 'rev1', 'vendor': 'BIOSVendor', 'version': 'v1.0'}, 'os': {'codename': 'focal', 'description': 'Ubuntu 20.04.1 LTS', 'distributor': 'Ubuntu', 'kernel': {'name': 'Linux', 'signature': 'Sample Signature', 'version': '5.4.0-42-generic'}, 'loaded_modules': ['module1', 'module2'], 'version': '20.04'}, 'status': 'Certified'}
```


## Run tests

Since we're using python bindings, this library contains tests for both Rust and Python code.
To execute them, run the following commands in the `hwlib/` directory:

* Run Rust tests: `$ cargo test -- --test-threads=1`
* For Python tests, you need to have `tox` on your system installed: `pip install tox`.
Then, you can run Python tests with tox `$ tox`


## Build deb package

This section describes how to pack `hwlib` as a debian package. Before getting started, make sure that you have the following packages installed on your system:

```bash
sudo apt-get install -y debhelper dh-cargo devscripts
```

### Building the rust lib

First, generate update to the changelog file using the `dch` tool:

```bash
cd client/hwlib
export DEBEMAIL=your.email@canonical.com
export DEBFULLNAME="Name Surname"
dch -i  # increment release number
dch -r  # create release
```


Then, build the source package:

```bash
dpkg-buildpackage -S #-k=<key-to-sign> if you have more than one GPG key for the specified DEBEMAIL
```


### Testing your package

You can test your package and build it with the [sbuild](https://wiki.debian.org/sbuild) tool.
In this example, we do it for focal distro, but you can replace it with the desired one:

```bash
sudo apt install sbuild mmdebstrap uidmap
mkdir -p ~/.cache/sbuild
mmdebstrap --variant=buildd --components=main,restricted,universe focal /home/zyga/.cache/sbuild/focal-amd64.tar.zst
```

For configuring `sbuild` , install `sbuild-debian-developer-setup`:

```bash
sudo apt install sbuild-debian-developer-setup
sudo sbuild-debian-developer-setup
newgrp sbuild
```

Update chroot:

```bash
sudo sbuild-update -udcar u
schroot -l | grep sbuild
```

Put the following content into `~/.sbuildrc` file:

```perl
$chroot_mode = 'unshare';
$run_autopkgtest = 0;
$autopkgtest_root_args = '';
$autopkgtest_opts = [ '--apt-upgrade', '--', 'unshare', '--release', '%r', '--arch', '%a' ];
```

Not you can build the binary itself:

```bash
sbuild /path/to/.dsc -d focal
```

After that, you can publish the package by rinning:

```bash
 dput ppa:<ppa_name> ../<package>_<version>_source.changes
 ```
