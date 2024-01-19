# Hardware Information API (hi-api)

The repo contains the API server and client for retrieving hardware information.

## Build and run the server

```bash
poetry install
poetry run uvicorn hi_api.main:app —reload
```

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
