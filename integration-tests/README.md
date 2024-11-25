# Integration tests for the project

This folder contains the tests for the hardware-api project that check
how client and server communicate. They use the real data from
specific certified machines and make sure that this hardware is still
indicated as certified regardless the changes in the client or server
code.

The tests require access to https://certification.canonical.com
service. To run them, first install
[docker](https://docs.docker.com/engine/install/ubuntu/) and [setup
permissions](https://docs.docker.com/engine/install/linux-postinstall/). Then,
run the tests with `docker compose`:

```sh
docker compose up --build --abort-on-container-exit
```
