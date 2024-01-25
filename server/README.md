# hwapi server


## Installation

This package just requires `poetry` to be installed on your system, so all the dependencies are managed by it. Go to the `server/` directory in the project and run the following commands:

```bash
$ poetry install
$ poetry run uvicorn hwapi.main:app --reload
```

Then you can access the server via this URL: http://127.0.0.1:8000


## Accessing API schema

To access the HTML view for the API schema, just run the server and follow the [/#docs](http://127.0.0.1:8000/#docs) endpoint.

For getting its JSON version, follow the [/openapi.json](http://127.0.0.1:8000/openapi.json) endpoint. 

You can retrieve the schema in the YAML format by following the [/openapi.yaml](http://127.0.0.1:8000/v1/openapi.yaml) endpoint ior just reading the [openapi.yaml](./openapi.yaml) file in the repo.

## Development

### Pre-commit hooks

The repo contains pre-commit hook rules to update the openapi.yaml file before committing the changes. To use it, first make sure `pre-commit` is installed on your system (is installed with poetry dev dependencies).

Then go to the `server/` directory and run `poetry run pre-commit install`.
