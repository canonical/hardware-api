# `hwlib` library and `hwctl` CLI tool

The client contains of two modules:

* [`hwlib`](./hwlib): Rust library for communicating with the API
  server
* [`hwctl`](./hwctl): CLI tool (written in Rust) that provides a user
  with the CLI tool for the `hwlib`

## Build package in offline mode with vendored dependencies

To build the package using the vendored dependencies and then run it
in offline mode, execute the following commands:

```bash
cd client/hwlib
# By default, the vendored dependencies are stored under the ./vendor/ directory
./debian/vendor-rust.sh
```

Then, modify your `~/.cargo/config.toml` (or `~/.cargo/config` if you
use an older cargo version) to use the vendored dependencies. Don't
forget to specify the correct path:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/path/to/hardware-api/client/hwlib/vendor"
```

Currently, there is no way to change these settings per project, so
cargo will use these vendored dependencies for all your Rust
projects. You can remove these lines after finishing the work on the
`hwlib`.

Now you can build the project in offline mode:

```bash
cd client/
cargo build --offline
```

## Use Python bindings

The `hwlib` lib can be used in Python code as well. We're using
[pyo3](https://github.com/PyO3/pyo3) lib for creating Python bindings,
so to build them, you need to have
[maturin](https://github.com/PyO3/maturin) on your system
installed. It requires virtual environment to be configured to work
with it:

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

Now you can use the lib in your Python code. The library requires root
access since we collect the hardware information using SMBIOS data. If
you're running it on a device that doesn't have SMBIOS data available,
root privileges are not required.

```python
$ sudo /path/to/venv/bin/python3  # or `python3`
>>> import hwlib
>>> hwlib.get_certification_status("https://hw.ubuntu.com")
```


## Run tests

Since we're using python bindings, this library contains tests for
both Rust and Python code.  To execute them, run the following
commands in the `hwlib/` directory:

* Run Rust tests: `$ cargo test`
* For Python tests, you need to have `tox` on your system installed:
`pip install tox`.  Then, you can run Python tests with tox `$ tox`


## Build deb package

This section describes how to pack `hwlib` as a debian package. Before
getting started, make sure that you have the following packages
installed on your system:

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

Then copy the Cargo.lock file from the project root to `client/hwlib`
dir, because the MIR policy requires the lock file to be included to
the archive

```bash
cp ../../Cargo.lock ./
```

Then you need to vendor the Rust dependencies:

```bash
# under client/hwlib/ dir
./debian/vendor-rust.sh
```

If the cargo dependencies got updated, you also need to update the
`XS-Vendored-Sources-Rust` header. The value can be retrieved from the
`expected` section of the output of this command:

```sh
export CARGO_VENDOR_DIR=$(pwd)/rust-vendor/
/usr/share/cargo/bin/dh-cargo-vendored-sources
```

`dh-cargo` requires the `debian/cargo-checksum.json` file to be
present in the archive. Until the package is not published to
crates.io, we need to generate it ourselves:

```bash
# under client/hwlib/ dir
./debian/generate_checksums.py
```

Then, build the source package:

```bash
dpkg-buildpackage -S #-k=<key-to-sign> if you have more than one GPG key for the specified DEBEMAIL
```

You can also `lintian --pedantic` to staticly check the files under
the `debian/` dir.

### Testing the package build

You can test your package and build it with the
[sbuild](https://wiki.debian.org/sbuild) tool.  In this example, we do
it for oracular distro, but you can replace it with the desired one:

```bash
sudo apt install sbuild mmdebstrap uidmap
mkdir -p ~/.cache/sbuild
mmdebstrap --variant=buildd --components=main,restricted,universe oracular ~/.cache/sbuild/oracular-amd64.tar.zst
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
sbuild /path/to/.dsc -d oracular
```


### Running autopkgtests locally in lxd

To run autopkgtests, first set up the environment. It can be set up by
running the following command (the distro can be different):

```sh
autopkgtest-build-lxd ubuntu-daily:noble/amd64
```

Then, go to the `client/hwlib` directory and run the autopkgtests in
`lxd`:

```sh
autopkgtest . -- lxd autopkgtest/ubuntu/noble/amd64
```

### Publishing the package

After the archive is created and you've tested the build locally, you
can publish the package by running:

```sh
dput ppa:<ppa_name> ../<package>_<version>_source.changes
 ```
