# Contributing to Hardware API

Thanks for taking the time to contribute to Hardware API.

Here are the guidelines for contributing to the development of Hardware API.
These are guidelines, not hard and fast rules; but please exercise judgement.
Feel free to propose changes to this document.

Hardware API is hosted and managed on [GitHub]. If you're not familiar with
how GitHub works, their [quickstart documentation][github-quickstart] provides
an excellent introduction to all the tools and processes you'll need to know.

Pull requests are merged with rebasing, so please make sure your commits are
atomic and well described. We generally follow the specification of
[Conventional Commits].

For server-specific contribution guidelines, refer to the
[server contribution guide](./server/CONTRIBUTING.md).

For client-specific contribution guidelines, refer to the
[client contribution guide](./client/CONTRIBUTING.md).

## Pre-requisites

Before you can begin, you will need to:

- Read and agree to abide by our [Code of Conduct][code-of-conduct].
- Create (or have) a GitHub account.
- Sign the Canonical [contributor license agreement][cla]. This grants us your
  permission to use your contributions in the project.

## Recommendations

## Task runner

The project uses [just] as a task runner and provides `justfile`s for each
sub-project.

You can install just as a [snap][just-snap] or with `uv`:

```shell
uv tool install rust-just
```

Then run `just` from anywhere in the repository for usage.

## Pre-commit

The project provides [pre-commit] configuration files (`.pre-commit-config.yaml`).
However, [prek] is preferred, as it supports the monorepo/workspace layout of
the project.

You can install the pre-commit hooks with the following `just` recipe:

```shell
just pre-commit
```

## Workshop

The project provides a [Workshop] definition to stand up a development
environment. First install the `workshop` snap:

```shell
# Workshop requires LXD 6.8+, so make sure to install/update LXD:
# To install: `sudo snap install --channel 6/stable lxd`
# To update: `sudo snap refresh --channel 6/stable lxd`
sudo snap install --classic workshop
```

Then you can launch the workshop:

```shell
workshop launch
```

Now you can:

- List the provided actions: `workshop actions`
- Run a provided action: `workshop run ...`
- Connect to workshop: `workshop shell`
- Execute a command in the workshop: `workshop exec ...`

To learn more, refer to the [Workshop documentation].

## Local Deployment

- Install [Docker] and [set up permissions][docker-permissions] for the server
  deployment. For further information on the server deployment, refer to the
  [server contribution guide](./server/CONTRIBUTING.md).
- [Install `rust` and `cargo`][rust-install] for client deployment. For further
  information on the client deployment, refer to the
  [client contribution guide](./client/CONTRIBUTING.md).

To run the server locally, make sure that no other application is using port
`8080`.

[github]: https://github.com/canonical/hardware-api
[github-quickstart]: https://docs.github.com/en/get-started/quickstart
[conventional commits]: https://www.conventionalcommits.org/en/v1.0.0/
[code-of-conduct]: https://ubuntu.com/community/code-of-conduct
[cla]: https://ubuntu.com/legal/contributors
[just]: https://github.com/casey/just
[pre-commit]: https://pre-commit.com/
[prek]: https://github.com/j178/prek
[just-snap]: https://snapcraft.io/just
[workshop]: https://github.com/canonical/workshop
[workshop documentation]: https://ubuntu.com/workshop/docs/
[docker]: https://docs.docker.com/engine/install/ubuntu/
[docker-permissions]: https://docs.docker.com/engine/install/linux-postinstall/
[rust-install]: https://www.rust-lang.org/tools/install
