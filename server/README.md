# hwapi server


## Development Installation

This package just requires `poetry` to be installed on your system, so all the dependencies are managed by it. Go to the `server/` directory in the project and run the following commands:

```bash
$ poetry install
$ poetry run uvicorn hwapi.main:app --reload
```

Then you can access the server via this URL: http://127.0.0.1:8000


## Pre-commit hooks

The repo contains pre-commit hook rules to update the openapi.yaml file before committing the changes. To use it, first make sure `pre-commit` is installed on your system (is installed with poetry dev dependencies).

For generating OpenAPI schema in HTML format, you need Node.js to be installed on your system:

```bash
# Install Node.js v20
$ curl -sL https://deb.nodesource.com/setup_20.x | sudo -E bash -
$ sudo apt-get install -y nodejs
```

Then go to the `server/` directory and run `poetry run pre-commit install`.

