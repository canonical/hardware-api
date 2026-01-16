import json

import pytest


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
