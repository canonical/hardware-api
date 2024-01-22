# Hardware Information API (hi-api)

The repo contains the API server and client for retrieving hardware information.

## Build and run the server

```bash
poetry install
poetry run uvicorn hi_api.main:app --reload
```

## View OpenAPI schema for the server

A HTML view of the OpenAPI schema matching the `main` branch is viewable at
[canonical.github.io/hardware-api](https://canonical.github.io/hardware-api)

## Build and run the reference CLI tool (`hwctl`)

```bash
cd client/hwctl
cargo run
```

## Build the library (`hwlib`)

```bash
cd client/hwlib
cargo build
```
