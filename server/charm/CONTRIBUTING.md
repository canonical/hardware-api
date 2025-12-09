# Contributing to Hardware API Charm

To make contributions to this charm, you'll need a working
[development setup][juju-setup].

You can create an environment for development with [`uv`][uv] (you can install it as a [snap][uv-snap]):

```shell
uv sync --all-groups
source .venv/bin/activate
```

## Testing

This project uses [`tox`][tox] for managing test environments. There are some pre-configured environments
that can be used for linting and formatting code when you're preparing contributions to the charm:

```shell
tox run -e format        # update your code according to linting rules
tox run -e lint          # code style
tox run -e unit          # unit tests
tox run -e integration   # integration tests
tox                      # runs 'format', 'lint', and 'unit' environments
```

## Build the charm

Build the charm with [`charmcraft`][charmcraft-snap]:

```shell
charmcraft pack
```

[juju-setup]: https://documentation.ubuntu.com/juju/3.6/howto/manage-your-deployment/#set-up-your-deployment-local-testing-and-development
[uv]: https://docs.astral.sh/uv
[uv-snap]: https://snapcraft.io/astral-uv
[tox]: https://tox.wiki/en/4.32.0/
[charmcraft-snap]: https://snapcraft.io/charmcraft
