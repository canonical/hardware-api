# Hardware API Integration Tests

This repository contains integration tests for the hardware-api
project, validating the communication between client and server
components using sanitized data from certified machines.

## Overview

The tests ensure that specific machines remain indentified as
certified regardless of changes in the client or server codebase. They
use real-world data from certified machines, sanitized for testing
purposes.

## Prerequisites

1. Install [Docker](https://docs.docker.com/engine/install/ubuntu/)
2. Configure [Docker
   permissions](https://docs.docker.com/engine/install/linux-postinstall/)

## Running the Tests

Execute the test suite using Docker Compose:

```bash
docker compose up --build --abort-on-container-exit
```

## Test Architecture

The test suite runs in Docker containers and consists of two main
components:

* Server Container:
  - Built using the [server Dockerfile](../server/Dockerfile)
  - Populated with test data using mocked
    [C3](https://certification.canonical.com) API responses
  - Mock data location:
    [server/scripts/c3\_test\_data/](../server/scripts/c3_test_data/)

* Client Container:
  - Uses `hwlib` to collect hardware information
  - Sends requests to the server container
  - Validates responses against expected results
  - Test data location:
    [client/test\_data/](../client/test_data/)

Test Flow:

1. Server container starts and populates the DB using mocked C3
   responses
2. Client container:
   - Collects hardware information using `hwlib`
   - Sends requests to the server
   - Compares responses with expected results in `response.json` files
3. Tests pass if all responses match their expected values

Test Data:

- Server mock data:
  [server/scripts/c3\_test\_data/](../server/scripts/c3_test_data/)
- Client test data:
  [client/test\_data/](../client/test_data/)
- Expected responses: `response.json` files in test directories

## Contributing

When adding new test machines data:

1. Add corresponding mock data to the server
2. Add test data to the client
3. Create appropriate `response.json` files
4. Update tests as needed
