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
# By default, the vendored dependencies are stored under the ./rust-vendor/ directory
./debian/vendor-rust.sh
```

Then, modify your `~/.cargo/config.toml` (or `~/.cargo/config` if you
use an older cargo version) to use the vendored dependencies. Don't
forget to specify the correct path:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/path/to/hardware-api/client/rust-vendor"
```

Currently, there is no way to change these settings per project, so
cargo will use these vendored dependencies for all your Rust
projects. You can remove these lines after finishing the work on the
`hardware-api` client project.

Now you can build the project in offline mode:

```bash
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

## Refreshing cargo dependencies

To update dependency versions, simply run `cargo update`. However,
this also requires updating the `XS-Vendored-Sources-Rust` header in the
`debian/control` file. You can get the new header value from the
expected section of this command's output:

```sh
./debian/vendor-rust.sh
export CARGO_VENDOR_DIR=$(pwd)/rust-vendor/
/usr/share/cargo/bin/dh-cargo-vendored-sources
```

If the command returns a non-zero exit code, you'll need to update the
`XS-Vendored-Sources-Rust` header with the value shown in the output.


## Build Debian package

This section describes how to pack `hwlib` as a debian package. Before
getting started, make sure that you have the following packages
installed on your system:

```bash
sudo apt-get install -y debhelper dh-cargo devscripts
```

### Creating a new version

To create a new version of the client package, follow these steps:

1. Update the Changelog
Generate update to the changelog file using the `dch` tool:

```bash
export DEBEMAIL=your.email@canonical.com
export DEBFULLNAME="Your Full Name"
dch -i  # increment release number
dch -r  # create release
```

2. Versioning Scheme

We follow the `MAJOR.MINOR.PATCH` semantic versioning convention for
this package:

* `MAJOR`: Incremented for incompatible API changes or significant functionality updates.
* `MINOR`: Incremented for backward-compatible feature additions.
* `PATCH`: Incremented for backward-compatible bug fixes.


For pre-releases targeted at a PPA, append the `~ppaN` suffix to the
version, where `N` represents the build number (e.g., `1.2.3~ppa1`). Once
the package is approved for publication to the main repository, remove
the `~ppaN` suffix to finalize the version.

3. Create a Matching Git Tag

Ensure the version is correctly tagged in Git. The tag should follow
the format `vMAJOR.MINOR.PATCH` and must be annotated to the same
commit as the changelog entry:

```
git tag -a v1.2.3 <commit_hash>
git push origin v1.2.3
```

4. Update Cargo Versions

If the new version includes changes to the MAJOR, MINOR, or PATCH
version numbers, update the version fields in the following files:

* `hwlib/Cargo.toml`
* `hwctl/Cargo.toml`

Make sure to commit these changes along with the updated changelog.

### Building the client package

After generating a new version, you need to vendor the Rust
dependencies:

```bash
./debian/vendor-rust.sh
```

`dh-cargo` requires the `debian/cargo-checksum.json` file to be
present in the archive. Until the package is not published to
crates.io, we need to generate it ourselves:

```bash
./debian/generate_checksums.py
```

Then, build the source package:

```bash
dpkg-buildpackage -S #-k=<key-to-sign> if you have more than one GPG key for the specified DEBEMAIL
```

You can also `lintian --pedantic` to statically check the files under
the `debian/` dir.

### Testing the package build

You can test your package and build it with the
[sbuild](https://wiki.debian.org/sbuild) tool. In this example, we do
it for plucky distro, but you can replace it with the desired one:

```bash
sudo apt install sbuild mmdebstrap uidmap
mkdir -p ~/.cache/sbuild
mmdebstrap --variant=buildd --components=main,restricted,universe plucky ~/.cache/sbuild/plucky-amd64.tar.zst
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
sbuild /path/to/.dsc -d plucky
```

### Running autopkgtests locally in lxd

To run autopkgtests, first set up the environment. It can be set up by
running the following command (the distro can be different):

```sh
autopkgtest-build-lxd ubuntu-daily:plucky/amd64
```

Then run the autopkgtests in `lxd`:

```sh
autopkgtest . -- lxd autopkgtest/ubuntu/plucky/amd64
```

### Publishing the package

After the archive is created and you've tested the build locally, you
can publish the package by running:

```sh
dput ppa:<ppa_name> ../<package>_<version>_source.changes
 ```
