# Preface

The Hardware API client is a Rust library, daemon, and CLI for checking hardware certification status. It is relevant for tasks involving CLI development, hardware data collection, Varlink protocol, snap packaging, Python bindings, or client-side caching.

Read the top-level `.kb/agents.md` file before continuing below.

# Overview

The Hardware API Client provides tools to check the certification status of hardware configurations. It consists of three crates: `hwlib` (Rust library with Python bindings via pyo3/maturin), `hwctl-daemon` (background service using the Varlink protocol), and `hwctl` (CLI tool communicating with the daemon over Varlink). The client collects local hardware data (CPU info, SMBIOS, OS info) and queries the server API, with support for local caching. It is published as a snap (`hwctl`), a Rust crate on crates.io, and a Python wheel.

# Architecture

The client follows a three-layer architecture:

- `hwlib` (`src/lib.rs`): Core Rust library with Python bindings (`py_bindings.rs`). It handles hardware data collection (`collectors/`), data models (`models/`), server communication, and cache management (`cache.rs`).
- `hwctl-daemon` (`src/bin/hwctl-daemon.rs`): Background service implementing the Varlink protocol defined in `com.ubuntu.hwctl.varlink`. It exposes `GetCertificationStatus` and `SetRemoteAccess` methods over a Unix domain socket.
- `hwctl` (`src/bin/hwctl.rs`): CLI tool communicating with the daemon. Supports `--origin` (auto/server/cache), `--server` URL override, and `--enable/disable-server-access` flags.

The Varlink socket is at `/var/snap/hwctl/common/hwctl.varlink` for snaps and `/run/hwctl/hwctl.varlink` for unconfined installations.

# Directory

- `src/` - Rust source (library, CLI binary, daemon binary, Varlink protocol)
- `src/collectors/` - Hardware data collectors (CPU, SMBIOS, OS info)
- `src/models/` - Data models (devices, software, request/response validators)
- `pytests/` - Python-based tests for certification status
- `test_data/` - Test device directories with hardware profiles
- `cache/` - Default cached hardware data
- `snap/` - Snap package hooks
- `dist/` - Built Python wheels and source distributions
- `Cargo.toml` - Rust project configuration
- `pyproject.toml` - Python build configuration (maturin)
- `snapcraft.yaml` - Snap package definition (core24, strict confinement)
- `justfile` - Client task runner (setup, format-rs, format-py, lint-rs, lint-py, test, snapcraft)
- `tox.ini` - Tox config for Python tests (format, lint, unit)

# Documents

- `.kb/testing.md` - Client testing procedures and `just` commands.
