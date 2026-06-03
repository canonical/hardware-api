# Client-specific recipes.
mod client
# Server-specific recipes.
mod server
# Documentation-specific recipes.
mod docs

[doc('Describe usage and list the available recipes.')]
help:
    @echo '{{ BOLD }}Canonical Hardware API Project{{ NORMAL }}'
    @echo
    @just --list --unsorted

[doc('Perform static analysis on GitHub workflows.')]
zizmor:
    uvx zizmor --pedantic .

[doc('Format all projects.')]
format:
    @just client::format server::format server::charm::format

[doc('Lint all projects.')]
lint:
    @just client::lint server::lint server::charm::lint

[doc('Run all unit tests.')]
test:
    @just client::test server::test server::charm::test

[doc('Run integration tests.')]
[working-directory('integration-tests')]
integration:
    docker compose up --build --abort-on-container-exit
