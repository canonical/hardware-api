# hwapi server


## Installation

   This package just requires `poetry` to be installed on your system, so all the dependencies are managed by it. Go to the `server/` directory in the project and run the following commands:

```bash
$ poetry install
$ poetry run uvicorn hwapi.main:app --reload
```

Then you can access the server via this URL: http://127.0.0.1:8000


## Accessing API schema

To access the HTML view for the API schema, just run the server and follow the `/#docs` endpoint.

For getting its JSON version, follow the `/openapi.json` endpoint.
