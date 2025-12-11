# Hardware API Charm

[![Charmcraft][charmcraft-badge]][charmcraft-site]
[![Test API server][test-badge]][test-site]
[![codecov][cov-badge]][cov-latest]
[![uv Status][uv-badge]][uv-site]
[![Ruff status][ruff-badge]][ruff-site]

**Hardware API Charm** operates one or more units of the [Hardware API Server](../README.md).

## Basic Usage

On [Juju-ready][juju] systems, you can deploy it on the command-line with:

```shell
juju deploy hardware-api
```

## Configuration

- `log-level` (default: `info`): Allows you to set the log level for the server.
- `port` (default: `30000`): Port to listen on for incoming HTTP requests.
- `hostname` (default: `hw`): External hostname for the service.

## Relations / Integrations

Currently, supported relations are:

- `nginx-route`: for interfacing with [`nginx-ingress-integrator`][nginx-ingress-integrator].

## Community and Support

You can report any issues, bugs, or feature requests on the project's
[GitHub repository][canonical/hardware-api].

## Contribute to the Hardware API Charm

The Hardware API Charm is open source. Contributions are welcome.

If you're interested, start with the
[charm contribution guide](CONTRIBUTING.md).

## License and Copyright

The Hardware API Charm is released under the [Apache-2.0 license](LICENSE).

© 2025 Canonical Ltd.

[charmcraft-badge]: https://charmhub.io/hardware-api/badge.svg
[charmcraft-site]: https://charmhub.io/hardware-api
[test-badge]: https://github.com/canonical/hardware-api/actions/workflows/test_server.yaml/badge.svg
[test-site]: https://github.com/canonical/hardware-api/actions/workflows/test_server.yaml
[cov-badge]: https://codecov.io/gh/canonical/hardware-api/graph/badge.svg?token=p0h9tTp2F3&component=server&flags=charm
[cov-latest]: https://codecov.io/gh/canonical/hardware-api
[uv-badge]: https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/uv/main/assets/badge/v0.json
[uv-site]: https://github.com/astral-sh/uv
[ruff-badge]: https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json
[ruff-site]: https://github.com/astral-sh/ruff
[juju]: https://canonical.com/juju
[canonical/hardware-api]: https://github.com/canonical/hardware-api
[nginx-ingress-integrator]: https://charmhub.io/nginx-ingress-integrator
