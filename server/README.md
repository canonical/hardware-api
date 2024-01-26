# hwapi server


## Installation

This package just requires `poetry` to be installed on your system, so all the dependencies are managed by it. Go to the `server/` directory in the project and run the following commands:

```bash
$ poetry install
$ poetry run uvicorn hwapi.main:app --reload
```

Then you can access the server via this URL: http://127.0.0.1:8000


## Accessing API schema

You can retrieve API schema in HTML, YAML, and JSON formats:

- To access the HTML view for the API schema, just run the server and follow the [/#docs](http://127.0.0.1:8000/#docs) endpoint.
- For getting its JSON version, follow the [/openapi.json](http://127.0.0.1:8000/openapi.json) endpoint.
- Retrieve the schema in YAML from the running service by following the [/openapi.yaml](http://127.0.0.1:8000/v1/openapi.yaml) endpoint
- A copy of the [openapi.yaml](./hwapi/schemas/openapi.yaml) is included in the repo, and it is enforced by a CI automation to be up to date.
- A self-contained HTML representation of the schema is also included in the repository: [openapi.html](./hwapi/schemas/openapi.html).

## Development

### Pre-commit hooks

The repo contains pre-commit hook rules to update the openapi.yaml file before committing the changes. To use it, first make sure `pre-commit` is installed on your system (is installed with poetry dev dependencies).

For generating OpenAPI schema in HTML format, you need Node.js to be installed on your system:

```bash
# Install Node.js v20
$ curl -sL https://deb.nodesource.com/setup_20.x | sudo -E bash -
$ sudo apt-get install -y nodejs
```

Then go to the `server/` directory and run `poetry run pre-commit install`.
