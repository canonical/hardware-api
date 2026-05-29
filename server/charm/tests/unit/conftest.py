# Copyright (C) 2026 Canonical Ltd.
"""Testing fixtures."""

from dataclasses import dataclass

import pytest
from ops import testing

from charm import CONTAINER_NAME, HardwareApiCharm


@pytest.fixture(scope="function")
def ctx() -> testing.Context:
    """Create the Hardware API Charm context."""
    return testing.Context(HardwareApiCharm)


@dataclass
class DatabaseFixture:
    """Holds the state needed to simulate the database (postgresql_client) relation."""

    relation: testing.Relation
    username: str
    password: str
    endpoints: str
    resource: str

    @property
    def url(self) -> str:
        """The database URL the charm is expected to build from the relation."""
        return f"postgresql://{self.username}:{self.password}@{self.endpoints}/{self.resource}"


@pytest.fixture(scope="function")
def database() -> DatabaseFixture:
    """Create a database relation fixture."""
    username = "relation-3"
    password = "supersecret"
    endpoints = "10.180.162.1:5432"
    resource = "hwapi"

    remote_app_data = {
        "endpoints": endpoints,
        "username": username,
        "password": password,
    }
    relation = testing.Relation(
        "database",
        interface="postgresql_client",
        remote_app_name="postgresql",
        remote_app_data=remote_app_data,
        remote_units_data={0: {}},
    )
    return DatabaseFixture(
        relation=relation,
        username=username,
        password=password,
        endpoints=endpoints,
        resource=resource,
    )


@pytest.fixture(scope="function")
def container() -> testing.Container:
    """Create a container for testing."""
    return testing.Container(name=CONTAINER_NAME, can_connect=True)


@pytest.fixture(scope="function")
def ingress_relation() -> testing.Relation:
    """Create an ingress relation for testing."""
    return testing.Relation("ingress", remote_app_data={"hostname": "hw.local"})
