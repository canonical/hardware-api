# Preface

This document outlines the testing strategy for the Hardware API monorepo. Testing is split into server-specific and client-specific concerns, with each sub-project documenting its own testing procedures.

Read the top-level `.kb/agents.md` file before continuing below.

# Overview

The Hardware API project defines tests at three levels:

- **Unit tests** – per sub-project, run via `just test` in each directory.
- **Integration tests** – Docker-based end-to-end tests validating client-server communication with sanitized certified-machine data.
- **Static analysis** – GitHub workflow analysis with `zizmor`.

# Important

Always use `just` commands, not the underlying tool invocations. Run linting and formatting before submitting code.

- `just test` – runs all unit tests (client, server, charm).
- `just lint` – runs all linting (client, server, charm).
- `just format` – runs all formatting (client, server, charm).
- `just zizmor` – static analysis on GitHub workflows.
- `just integration` – Docker-based integration tests.

If any tests or commands are missing a `just` recipe, add it to the `justfile`.

Server- and client-specific details are in their respective `.kb/testing.md` files.
