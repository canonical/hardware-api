# Copyright (C) 2026 Canonical Ltd.
"""Testing fixtures."""

import json

import pytest
from ops import testing

from charm import HardwareApiCharm


@pytest.fixture(scope="function")
def ctx() -> testing.Context:
    """Create the Hardware API Charm context."""
    return testing.Context(HardwareApiCharm)


@pytest.fixture(scope="function")
def ingress_template() -> str:
    """Create a configuration for the traefik-k8s ingress."""
    template = {
        "model": "{{ model }}",
        "app": "{{ app }}",
        "public_port": "{{ public_port }}",
        "external_hostname": "{{ external_hostname }}",
    }
    return json.dumps(template)
