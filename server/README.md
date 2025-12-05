# Hardware API Server

[![Charmcraft][charmcraft-badge]][charmcraft-site]
[![Documentation status][rtd-badge]][rtd-latest]
[![Test API server][test-badge]][test-site]
[![codecov][cov-badge]][cov-latest]
[![Poetry][poetry-badge]][poetry-site]
[![Ruff status][ruff-badge]][ruff-site]

The **Hardware API Server** is the microservice that provides an API to check
the certification status of hardware configurations.

## Basic Usage

The Hardware API Server is a [`fastapi`][fastapi] application.

You can run it (for example) with [`uvicorn`][uvicorn]:

```shell
uvicorn hwapi.main:app --host 0.0.0.0 --port 8080
```

## Installation

The Hardware API Server is available on all major Linux distributions.

On juju-ready systems, you can deploy it on the command-line with:

```shell
juju deploy hardware-api
```

## Community and Support

You can report any issues, bugs, or feature requests on the project's
[GitHub repository][github].

## Contribute to the Hardware API Server

The Hardware API Server is open source. Contributions are welcome.

If you're interested, start with the
[server contribution guide](./CONTRIBUTING.md).

## License and Copyright

The Hardware API Sserver is released under the [AGPL-3.0 license](./LICENSE).

© 2025 Canonical Ltd.

[charmcraft-badge]: https://charmhub.io/hardware-api/badge.svg
[charmcraft-site]: https://charmhub.io/hardware-api
[rtd-badge]: https://readthedocs.com/projects/canonical-hardware-api/badge/?version=latest
[rtd-latest]: https://canonical-hardware-api.readthedocs-hosted.com/latest/
[test-badge]: https://github.com/canonical/hardware-api/actions/workflows/test_server.yaml/badge.svg
[test-site]: https://github.com/canonical/hardware-api/actions/workflows/test_server.yaml
[cov-badge]: https://codecov.io/gh/canonical/hardware-api/graph/badge.svg?token=p0h9tTp2F3&component=server
[cov-latest]: https://codecov.io/gh/canonical/hardware-api
[poetry-badge]: https://img.shields.io/endpoint?url=https://python-poetry.org/badge/v0.json
[poetry-site]: https://python-poetry.org/
[ruff-badge]: https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json
[ruff-site]: https://github.com/astral-sh/ruff
[fastapi]: https://fastapi.tiangolo.com/
[uvicorn]: https://www.uvicorn.org/
[github]: https://github.com/canonical/hardware-api
