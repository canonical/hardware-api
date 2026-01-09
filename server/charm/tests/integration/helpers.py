"""Helper functions for integration testing."""

import json
from typing import cast

import jubilant


def all_active_idle(status: jubilant.Status, *apps: str) -> bool:
    """Check all units are active/idle."""
    return jubilant.all_active(status, *apps) and jubilant.all_agents_idle(status, *apps)


def get_ingress_endpoint(juju: jubilant.Juju, ingress_name: str, app_name: str) -> str:
    """Get the ingress endpoint."""
    task = juju.run(unit=f"{ingress_name}/0", action="show-proxied-endpoints")
    assert task.return_code == 0
    results = cast(dict[str, str], task.results)
    proxied_endpoints = json.loads(results["proxied-endpoints"])
    return proxied_endpoints[app_name]["url"]
