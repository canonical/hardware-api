# Copyright (C) 2026 Canonical Ltd.
import logging
import os
from pathlib import Path

import pytest

logger = logging.getLogger(__name__)


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
