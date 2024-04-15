# hwapi server


## Importing Data from C3

To import the hardware, releases, and certificates objects from C3, you can call the `v1/importers/import-certs` endpoint:

```bash
$ curl http://127.0.0.1:8080/v1/importers/import-certs -X POST
```

All the importers endpoints accept connections only from the hosts specified in the `INTERNAL_HOSTS` environemnt variable (the hosts shall be specific using "," separator, for example:

```bash
$ export INTERNAL_HOSTS=192.168.0.1,10.0.0.1,127.0.0.1,::1`)
```

To allow connections from all hosts, specify `*` as a value. The default value is `127.0.0.1,::1`.

## Running Tests with docker-compose

To run tests inside a Docker container (again, using `--build` when iterating on source code changes, to force the image to be rebuilt):

```bash
docker-compose up --attach-dependencies --force-recreate --abort-on-container-exit --build hwapi-test
```


## Pre-commit Hooks

The repo contains pre-commit hook rules to update the openapi.yaml file before committing the changes. To use it, first make sure `pre-commit` is installed on your system (is installed with poetry dev dependencies).

For generating OpenAPI schema in HTML format, you need Node.js to be installed on your system:

```bash
# Install Node.js v20
$ curl -sL https://deb.nodesource.com/setup_20.x | sudo -E bash -
$ sudo apt-get install -y nodejs
```

Then go to the `server/` directory and run `poetry run pre-commit install`.

