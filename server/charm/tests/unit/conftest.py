# Copyright (C) 2026 Canonical Ltd.
"""Testing fixtures."""

import pytest
from ops import testing

from charm import HardwareApiCharm


@pytest.fixture(scope="function")
def ctx() -> testing.Context:
    """Create the Hardware API Charm context."""
    return testing.Context(HardwareApiCharm)
