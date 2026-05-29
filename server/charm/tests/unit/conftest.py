# Copyright (C) 2026 Canonical Ltd.
"""Testing fixtures."""

import pytest
from ops import testing

from charm import CONTAINER_NAME, HardwareApiCharm


@pytest.fixture(scope="function")
def ctx() -> testing.Context:
    """Create the Hardware API Charm context."""
    return testing.Context(HardwareApiCharm)


@pytest.fixture(scope="function")
def container() -> testing.Container:
    """Create a container for testing."""
    return testing.Container(name=CONTAINER_NAME, can_connect=True)


@pytest.fixture(scope="function")
def ingress_relation() -> testing.Relation:
    """Create an ingress relation for testing."""
    return testing.Relation("ingress", remote_app_data={"hostname": "hw.local"})
