# Contributing to the Hardware API Client

For general contribution guidelines for the `hardware-api` project,
refer to the [contribution guide](../CONTRIBUTING.md).

## Build the Package

### Cargo Builds

To build the project with Rust, run the following:

```shell
sudo apt-get install -y pkgconf libssl-dev
cargo build
# to build only the library: cargo build --package=hwlib
# to build only the CLI tool: cargo build --package=hwctl
```

### Offline Mode

To build the package using the vendored dependencies and then run it in offline
mode, execute the following commands:

```shell
./debian/vendor-rust.sh
```

Then modify `~/.cargo/config.toml` (or `~/.cargo/config` if you use an older
cargo version) to use the vendored dependencies. Don't forget to specify the
correct path:

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/path/to/hardware-api/client/rust-vendor"
```

Currently there is no way to change these settings per project, so cargo will
use these vendored dependencies for all your Rust projects. You can remove these
lines after finishing the work on the Hardware API Client project.

Now you can build the project in offline mode:

```bash
cargo build --offline
```

### Snap Package

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

### Debian Package

Make sure you have the following packages installed on your system:

```shell
sudo apt-get install -y debhelper dh-cargo devscripts
```

#### Create a New Version

To create a new version of the client package, follow these steps:

1. Update the Changelog using the `dch` tool:

   ```shell
   export DEBEMAIL=your.email@canonical.com
   export DEBFULLNAME="Your Full Name"
   dch -i  # increment release number
   dch -r  # create release
   ```

2. Set version according to our versioning scheme. We follow the
   `MAJOR.MINOR.PATCH` semantic versioning convention for this packge:

   - `MAJOR`: Incremented for incompatible API changes or significant
     functionality updates.
   - `MINOR`: Incremented for backward-compatible feature additions.
   - `PATCH`: Incremented for backward-compatible bug fixes.

   For pre-releases targeted at a PPA, append the `~ppaN` suffix to the version,
   where `N` represents the build number (e.g., `1.2.3~ppa1`). Once the package
   is approved for publication to the main repository, remove the `~ppaN` suffix
   to finalize the version.

3. Create a matching git tag. Ensure the version is correctly tagged in git. The
   tag should follow the format `vMAJOR.MINOR.PATCH` and must be annotated to the
   same commit as the changelog entry:

   ```shell
   git tag -a v1.2.3 <commit_hash>
   git push origin v1.2.3
   ```

4. Update cargo versions. If the new version includes changes to the `MAJOR`,
   `MINOR`, or `PATCH` version numbers, update the `workspace.package.version`
   field in [`Cargo.toml`](./Cargo.toml). Make sure to commit these changes
   along with the updated changelog.

#### Build Client Debian Package

You need to vendor the Rust dependencies:

```shell
./debian/vendor-rust.sh
```

`dh-cargo` requires the `debian/cargo-checksum.json` file to be present in the
archive. Until the package is not published to crates.io, we need to generate
it ourselves:

```shell
./debian/generate_checksums.py
```

Then build the source package:

```shell
dpkg-buildpackage -S # -k=<key-to-sign> if you have more than one GPG key for the specified DEBEMAIL
```

You can also `lintian --pedantic` to statically check the files under
[`debian/`](./debian/).

#### Test Debian Package Build

You can test your package and ubild it with [`sbuild`][sbuild]. In this example,
we do it for the questing distro, but you can replace it with the desired one:

```shell
sudo apt-get install -y sbuild mmdebstrap uidmap
mkdir -p ~/.cache/sbuild
mmdebstrap --variant=buildd --components=main,restricted,universe questing ~/.cache/sbuild/questing-amd64.tar.zst
```

To configure `sbuild`, install `sbuild-debian-developer-setup`:

```shell
sudo apt-get install -y sbuild-debian-developer-setup
sudo sbuild-debian-developer-setup
newgrp sbuild
```

Update chroot:

```shell
sudo sbuild-update -udcar u
schroot -l | grep sbuild
```

Put the following content into `~/.sbuildrc`:

```perl
$chroot_mode = 'unshare';
$run_autopkgtest = 0;
$autopkgtest_root_args = '';
$autopkgtest_opts = [ '--apt-upgrade', '--', 'unshare', '--release', '%r', '--arch', '%a' ];
```

Now you can build the binary itself:

```shell
sbuild /path/to/.dsc -d questing
```

#### Run `autopkgtest` in VM

Make sure you have QEMU installed on your system.

First download the image (replace `questing` with a corresponding release):

```shell
autopkgtest-buildvm-ubuntu-cloud -r questing -v --cloud-image-url=http://cloud-images.ubuntu.com/daily/server
```

The image size may be too small, so you probably need to resize the disk and
give it more storage:

```shell
qemu-img resize autopkgtest-questing-amd64.img +15G
```

Then run `autopkgtest`. The setup command adds more space to the `tmpfs`
partition because `autopkgtest` for cargo uses the `/tmp` directory to build
the package.

```shell
autopkgtest \
  --apt-upgrade \
  --shell-fail \
  --output-dir=dep8-rust-hwlib \
  --setup-commands="mount -o remount,size=10G /tmp" \
  /path/to/<package>_<version>_source.changes \
  -- qemu --ram-size 4096 /var/lib/adt-images/autopkgtest-questing-amd64.img
```

#### Publish the Debian Package

After the archive is created and you've tested the build locally, you can
publish the package by running:

```shell
dput ppa:<ppa_name> /path/to/<package>_<version>_source.changes
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

## Refresh Cargo Dependencies

To update the dependencies' versiions, simply run `cargo update`.

However, this also requires updating the `XS-Vendored-Sources-Rust` header in
[`debian/control`](./debian/control). You can get the new header value from the
expected seciton of this command's output:

```shell
./debian/vendor-rust.sh
export CARGO_VENDOR_DIR=$(pwd)/rust-vendor/
/usr/share/cargo/bin/dh-cargo-vendored-sources
```

If the above command returns a non-zero exit code, you'll need to update the
`XS-Vendored-Sources-Rust` header with the value shown in the output.

[snap]: https://snapcraft.io/hwctl
[snapcraft]: https://github.com/canonical/snapcraft
[sbuild]: https://wiki.debian.org/sbuild
[pyo3]: https://github.com/PyO3/pyo3
[maturin]: https://github.com/PyO3/maturin
[tox]: https://github.com/tox-dev/tox
