# Preface

This document covers testing for the Hardware API client (Rust library, CLI, daemon, and Python bindings). It is relevant for tasks involving hardware data collection, Varlink protocol, caching, or snap packaging.

Read the top-level `.kb/agents.md` file before continuing below.

# Important

Always use `just` commands. Do not invoke `cargo test`, `cargo clippy`, `tox`, or other tools directly.

- `just test` – run all client unit tests (Rust and Python).
- `just lint` – lint all files (Rust fmt check, clippy, Python lint).
- `just format` – format all files (Rust fmt, Python format).
- `just unit` – run unit tests with coverage (outputs `coverage.xml`).
- `just check-lock` – verify Cargo lockfile integrity.

If any tests or commands are missing a `just` recipe, add it to the `justfile`.

# Architecture

Client unit tests are split across two ecosystems:

- **Rust tests** – under `src/`, run via `just test-rs`. These cover library logic, hardware collectors, data models, cache, and CLI/daemon binaries.
- **Python tests** – under `pytests/`, run via `just test-py`. These test the Python bindings and certification status logic using test data from `test_data/`.

Integration tests (at the monorepo level, `integration-tests/`) validate end-to-end client-server communication using sanitized certified-machine data and Docker Compose.
