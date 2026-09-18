# Preface

This document covers testing for the Hardware API server, including the Juju charm. It is relevant for tasks involving API endpoints, data models, external integrations, OpenAPI schema, or charm logic.

Read the top-level `.kb/agents.md` file before continuing below.

# Important

Always use `just` commands. Do not invoke `tox`, `pytest`, `terraform`, or other tools directly.

- `just test` – run server unit tests.
- `just lint` – lint Python files.
- `just format` – format Python files.
- `just check-schema` – validate the OpenAPI schema.
- `just charm::test` – run charm unit tests.
- `just charm::lint` – lint charm files.
- `just charm::format` – format charm files.
- `just charm::integration` – run charm integration tests (requires Juju environment).
- `just check-lock` – verify lockfile integrity.
- `just check-terraform` – lint and check Terraform module formatting and docs.

If any tests or commands are missing a `just` recipe, add it to the `justfile`.

# Architecture

Server unit tests live under `tests/` and use pytest with coverage reporting. Test data generator helpers are in `tests/conftest.py`. The OpenAPI schema is validated via a Tox `check-schema` environment.

Charm tests are under `charm/tests/` and include both unit tests (`unit` Tox environment) and integration tests (`integration` Tox environment) that require a Juju controller.
