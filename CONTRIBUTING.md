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
[docker]: https://docs.docker.com/engine/install/ubuntu/
[docker-permissions]: https://docs.docker.com/engine/install/linux-postinstall/
[rust-install]: https://www.rust-lang.org/tools/install
