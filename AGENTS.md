# Preface

Read the top-level `.kb/agents.md` file before continuing below.

# Overview

Hardware API is a monorepo for checking hardware certification status through a client-server architecture. The server (`server/`) is a Python/FastAPI microservice that queries the C3 (Canonical Certification Center) backend. The client (`client/`) is a Rust library (`hwlib`), daemon (`hwctl-daemon`), and CLI (`hwctl`) that collect local hardware information and communicate with the server. The project also includes a Juju charm for K8s deployment and Sphinx-based documentation.

# Architecture

The system follows a client-server model:

- **Docs** (`docs/`): Sphinx documentation hosted on ReadTheDocs.
- **Client** (`client/`): Rust library with Python bindings. The `hwctl` CLI communicates via Varlink with a local `hwctl-daemon` service that collects hardware info (CPU, SMBIOS, OS) and queries the server API. Results are cached locally.
- **Server** (`server/`): Python FastAPI application exposing endpoints to validate hardware certification. It uses SQLAlchemy with SQLite for data persistence, fetching data from C3. The server runs via Docker (uvicorn on port 8080) or juju charm on Kubernetes.
- **Charm** (`server/charm/`): Juju charm deploying the server on Kubernetes with Traefik ingress.

# Directory

- `server/` - Hardware API server (Python/FastAPI), charm, and Terraform module
- `client/` - Hardware API client (Rust library, CLI, daemon) with Python bindings
- `docs/` - Sphinx documentation site
- `integration-tests/` - Docker-based integration tests using the Rust client against a real server
- `justfile` - Root task runner with mod aliases for `client`, `server`, and `docs`
- `.workshop/` - Workshop dev environment definition

# Documents

- `.kb/agents.md` - General rules for the knowledge base reading and writing.
- `.kb/testing.md` - Testing strategy overview and top-level `just` commands.
- `client/AGENTS.md` - Hardware API client agentic context
- `server/AGENTS.md` - Hardware API server agentic context
