# hwapi server

## Running tests with docker-compose

To run tests inside a Docker container (again, using `--build` when iterating on source code changes, to force the image to be rebuilt):

```bash
docker compose up --attach-dependencies --force-recreate --abort-on-container-exit --build hwapi-test
```

## Pre-commit hooks

The repo contains pre-commit hook rules to update the openapi.yaml file before committing the changes. To use it, first make sure `pre-commit` is installed on your system (is installed with poetry dev dependencies).

For generating OpenAPI schema in HTML format, you need Node.js to be installed on your system:

```bash
# Install Node.js v20
curl -sL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
```

Then go to the `server/` directory and run `poetry run pre-commit install`.

