# Integration tests for the project

This folder contains the tests for the hardware-api project that check
how client and server communicate.

To run them, first install
[docker](https://docs.docker.com/engine/install/ubuntu/) and [setup
permissions](https://docs.docker.com/engine/install/linux-postinstall/).

Then, run the tests with `docker compose`:

```sh
docker compose up --build --abort-on-container-exit
```
