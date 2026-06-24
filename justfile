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

[doc('Format all projects.')]
[group('lint')]
format:
    @just client::format server::format server::charm::format

[doc('Lint all projects.')]
[group('lint')]
lint:
    @just client::lint server::lint server::charm::lint

[doc('Perform static analysis on GitHub workflows.')]
[group('test')]
zizmor:
    @uvx zizmor --pedantic .

[doc('Run all unit tests.')]
[group('test')]
test:
    @just client::test server::test server::charm::test

[doc('Run integration tests.')]
[group('test')]
[working-directory('integration-tests')]
integration:
    @docker compose up --build --abort-on-container-exit
