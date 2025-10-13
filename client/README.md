# Hardware API Client

[![Snapcraft][snapcraft-badge]][snapcraft-site]
[![Test client lib and CLI tool][test-badge]][test-site]

The **Hardware API Client** is the tool to check the certification status of
hardware configurations.

It consists of both the `hwlib` library and `hwctl` CLI tool.

- `hwlib`: Rust library for communicating with
  [the API server](../server/).
- `hwctl`: CLI tool leveraging the `hwlib` library.

## Basic Usage

To check the machine certification status, simply run `hwctl` with
root[^sudo] privileges:

```shell
sudo hwctl
```

To send the request to a different server than the [default][hw-server],
specify the `HW_API_URL` environment variable:

```shell
sudo HW_API_URL=https://your.server.url hwctl
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

## Community and Support

You can report any issues, bugs, or feature requests on the project's
[GitHub repository][github-issues].

## Contribute to the Hardware API Client

The Hardware API Client is open source. Contributions are welcome.

If you're interested, start with the
[client contribution guide](./CONTRIBUTING.md).

## License and Copyright

The Hardware API Client is released under the under the [LGPL-3.0 license](./LICENSE)

© 2025 Canonical Ltd.

[^sudo]:
    The client requires root access since we collect the hardware information
    using SMBIOS data.

[snapcraft-badge]: https://snapcraft.io/hwctl/badge.svg
[snapcraft-site]: https://snapcraft.io/hwctl
[test-badge]: https://github.com/canonical/hardware-api/actions/workflows/test_client.yaml/badge.svg
[test-site]: https://github.com/canonical/hardware-api/actions/workflows/test_client.yaml
[hw-server]: https://hw.ubuntu.com
[github-issues]: https://github.com/canonical/hardware-api/issues
