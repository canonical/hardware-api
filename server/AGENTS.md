# Preface

The Hardware API server is a Python/FastAPI microservice that exposes certification endpoints. It is relevant for tasks involving API development, certification logic, C3 data integration, database models, OpenAPI schema, charm deployment, or Terraform infrastructure.

Read the top-level `.kb/agents.md` file before continuing below.

# Overview

The Hardware API Server (`hwapi`) is a FastAPI application that provides an API to check the certification status of hardware configurations. It connects to the Canonical Certification Center (C3) for hardware certification data and uses SQLAlchemy with SQLite for persistence. The server is deployable via Docker, Juju charm, or Terraform.

# Architecture

The server follows a layered architecture under `src/hwapi/`:

- **Endpoints** (`endpoints/`): FastAPI route handlers. The `certification/` endpoint handles certification status queries, with separate modules for logic, request validation, response building, and response validation.
- **Data models** (`data_models/`): SQLAlchemy ORM models, a repository pattern for DB access, DB setup, and data validators for devices and software.
- **External integrations** (`external/`): HTTP clients for C3 and certified APIs, with URL definitions and response models.

The OpenAPI schema is maintained at `schemas/openapi.yaml` and generated from code via a Tox environment. Tests live under `tests/` and use pytest with coverage reporting.

A Juju charm (`charm/`) wraps the server for Kubernetes deployment with an ingress. A Terraform module (`terraform/`) provides infrastructure-as-code deployment.

# Directory

- `src/hwapi/` - Main Python package (FastAPI app, routers, endpoints, data models, external clients)
- `tests/` - Pytest-based unit tests with conftest and data generator
- `schemas/` - OpenAPI schema (`openapi.yaml`)
- `scripts/` - Utility scripts (C3 test data, schema generation, DB seeding, C3 import)
- `charm/` - Juju charm for Kubernetes deployment
- `terraform/` - Terraform module for Juju deployment
- `Dockerfile` - Docker image definition (uv-based, uvicorn on port 8080)
- `pyproject.toml` - Python project configuration (FastAPI, pydantic, SQLAlchemy, uvicorn)
- `justfile` - Server task runner (schema, format, lint, test, serve, teardown)
- `tox.ini` - Tox config (format, lint, unit, schema, check-schema)

# Documents

- `.kb/testing.md` - Server testing procedures and `just` commands.
