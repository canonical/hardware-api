# Hardware API Client

[![Snapcraft][snapcraft-badge]][snapcraft-site]
[![Crates.io][crate-badge]][crate-site]
[![Documentation status][rtd-badge]][rtd-latest]
[![Test client lib and CLI tool][test-badge]][test-site]
[![codecov][cov-badge]][cov-latest]

The **Hardware API Client** is the tool to check the certification status of
hardware configurations.

It consists of the `hwlib` library, the `hwctl-daemon` service and the `hwctl` CLI tool.

- `hwlib`: Rust library for communicating with
  [the API server](../server/) and to manage the certification cache.
- `hwctl-daemon`: Rust service leveraging the `hwlib` library.
- `hwctl`: CLI tool to retrieve the certification status and other data
  from the daemon.

## Basic Usage

To check the machine certification status, simply run `hwctl`:

```shell
hwctl
```

To send the request to a different server than the [default][hw-server],
you can either specify the `HW_API_URL` environment variable, or pass
the `--server` command line parameter:

```shell
hwctl --server https://your.server.url
```

You can specify whether you want to get the currently cached data, to
force a refresh from the server, or to let the application to decide
(return cached data if there are valid cached values, or get them from
the server otherwise). To do so, use the `--origin [auto|server|cache]`
parameter. For example:

```shell
hwctl --origin auto
```

The default value is `server`, if no parameter is specified, which will
always connect to the server to retrieve the data. The `cache` value will
always return the cached data (even if it is the first time and there is
no cache, in which case an `Unknown` state will be returned). Finally,
the `auto` will use several heuristics to decide whether to return the
cached values or connect to the server to refresh them.

Finally, you can enable or disable access to the server. If the access is
disabled, the daemon won't retrieve new data from the server when asked to
use the `auto` mode, only with the `server` mode. To disable the access,
use

```shell
hwctl --disable-server-access
```

and to enable it, use

```shell
hwctl --enable-server-access
```


## Installation

`hwctl` is available on all major Linux distributions.

On snap-ready systems, you can install it on the command-line with:

```shell
sudo snap install hwctl
```

On Questing Quokka (25.10), you can also install it using `apt`:

```shell
sudo apt-get install hwctl
```

## Service protocol

The hwctl service is socket-activated, and it uses [Varlink] as the underlying protocol.
The protocol is defined in the `bin/com.ubuntu.hwctl.varlink` file. It has two methods:

* GetCertificationStatus(source: CertificationSource, server_url: ?string) -> (state: State)
  It receives a `source` parameter, which can be any of `server`, `cache` or `auto`,
  and an optional (thus, it can be `null`) string with the certification server URL.
  It returns a State object with these fields:

  * status: `NotSeen`, `Certified`, `CertifiedImageExists`, `RelatedCertifiedSystemExists` or `Unknown`.
  * certified_url: either `null` if there is no applicable URL, or the URL describing either the current
    hardware, or the URL describing the related certified system.
  * available_releases: an array with the certified OS images available for this device, in the case of
    `CertifiedImageExists`. If not, it will be `null`.
  * valid_cache: a boolean that specifies whether the cache is valid or is old/not-existen/other problem.
  * hardware_mismatch: a boolean that specifies if the current hardware has changed respect to the one
    used to get the certified status. Only has meaning if the returned data was obtained from the cache.
  * stale: a boolean that is TRUE if the last connection to the server to update the certified status
    failed (and, thus, the current data is the previous cached one).
  * stale_reason: either `null`, or a string specifying why the last connection to the server failed.
  * source: either `Cache` or `Server`, specifying whether the data shown was obtained fresh from
    the server, or is the cached one.
  * remote_access_enabled: a boolean specifying if the service should refresh the cached data when
    using the `auto` source, or not.
  * server_url: a string with the server URL from which all the shown data was obtained in origin.

* SetRemoteAccess(enabled: bool) -> ()
  It receives a single boolean that changes the `remote_access_enabled` setting inside the service.

For snapped services, the socket path is `/var/snap/hwctl/common/hwctl.varlink`, while for
unconfined services, it is `/run/hwctl/hwctl.varlink`. Clients must always check the former
first, and only check for the later if the former doesn't exists.


## Community and Support

You can report any issues, bugs, or feature requests on the project's
[GitHub repository][github-issues].

## Contribute to the Hardware API Client

The Hardware API Client is open source. Contributions are welcome.

If you're interested, start with the
[client contribution guide](./CONTRIBUTING.md).

## License and Copyright

The Hardware API Client is released under the [LGPL-3.0 license](./LICENSE)

© 2025-2026 Canonical Ltd.

[snapcraft-badge]: https://snapcraft.io/hwctl/badge.svg
[snapcraft-site]: https://snapcraft.io/hwctl
[crate-badge]: https://img.shields.io/crates/v/hwlib.svg
[crate-site]: https://crates.io/crates/hwlib
[rtd-badge]: https://readthedocs.com/projects/canonical-hardware-api/badge/?version=latest
[rtd-latest]: https://canonical-hardware-api.readthedocs-hosted.com/latest/
[test-badge]: https://github.com/canonical/hardware-api/actions/workflows/test_client.yaml/badge.svg
[test-site]: https://github.com/canonical/hardware-api/actions/workflows/test_client.yaml
[cov-badge]: https://codecov.io/gh/canonical/hardware-api/graph/badge.svg?token=p0h9tTp2F3&component=client
[cov-latest]: https://codecov.io/gh/canonical/hardware-api
[hw-server]: https://hw.ubuntu.com
[github-issues]: https://github.com/canonical/hardware-api/issues
[Varlink]: https://varlink.org/
