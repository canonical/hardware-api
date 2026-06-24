# Copyright (C) 2026 Canonical Ltd.
import logging
import os
from pathlib import Path

import pytest
import pytest_jubilant

logger = logging.getLogger(__name__)


@pytest.fixture(scope="session")
def machine_controller():
    """Return the name of the machine controller to use for testing."""
    controller = os.environ.get("JUJU_MACHINE_CONTROLLER")
    if not controller:
        pytest.fail("JUJU_MACHINE_CONTROLLER environment variable is not set")
    return controller


@pytest.fixture(scope="module")
def machine_juju(juju_factory: pytest_jubilant.JujuFactory, machine_controller: str):
    """Create temporary Juju machine model for running tests."""
    yield juju_factory.get_juju(suffix="machine", controller=machine_controller)


@pytest.fixture(scope="session")
def charm():
    """Return the path of the charm under test."""
    if "CHARM_PATH" in os.environ:
        charm_path = Path(os.environ["CHARM_PATH"])
        if not charm_path.exists():
            raise FileNotFoundError(f"Charm does not exist: {charm_path}")
        return charm_path
    charm_paths = list(Path(".").glob("*.charm"))
    if not charm_paths:
        raise FileNotFoundError("No .charm file in current directory")
    if len(charm_paths) > 1:
        path_list = ", ".join(str(p) for p in charm_paths)
        raise ValueError(f"More than one .charm file in current directory: {path_list}")
    return charm_paths[0]
