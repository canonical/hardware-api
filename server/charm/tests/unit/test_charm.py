# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
#
# Learn more about testing at: https://juju.is/docs/sdk/testing
"""Unit tests for Hardware API Charm."""

import logging
from unittest.mock import MagicMock, mock_open, patch

import ops
import pytest
from ops import testing

from charm import HardwareApiCharm

logger = logging.getLogger(__name__)


def test_pebble_layer():
    """Tests that the Pebble layer is correctly set up."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "log-level": "info"},
        containers={container},
        leader=True,
    )
    state_out = ctx.run(ctx.on.pebble_ready(container), state_in)
    expected_plan = {
        "services": {
            "hardware-api": {
                "override": "replace",
                "summary": "Hardware API server",
                "command": "uvicorn hwapi.main:app --host 0.0.0.0 --port 30000 --log-level info",
                "startup": "enabled",
                # Since environment is empty, Layer.to_dict() omits it.
            }
        }
    }
    assert state_out.get_container(container.name).plan == expected_plan
    assert state_out.unit_status == testing.ActiveStatus()
    assert (
        state_out.get_container(container.name).service_statuses["hardware-api"]
        == ops.pebble.ServiceStatus.ACTIVE
    )


def test_config_changed_invalid_log_level():
    """Tests that an invalid log level in config_changed sets BlockedStatus."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "log-level": "invalid"},
        containers={container},
        leader=True,
    )
    with pytest.raises(testing.errors.UncaughtCharmError):
        state_out = ctx.run(ctx.on.config_changed(), state_in)
        assert isinstance(state_out.unit_status, testing.BlockedStatus)


def test_config_changed_pebble_not_ready():
    """Tests that config_changed defers event if Pebble is not ready."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=False)
    state_in = testing.State(
        config={"port": 30000, "log-level": "info"},
        containers={container},
        leader=True,
    )
    with patch("ops.framework.EventBase.defer") as patched_defer:
        ctx.run(ctx.on.config_changed(), state_in)
    patched_defer.assert_called_once()


def test_config_changed_updates_pebble_layer():
    """Tests that config_changed updates the Pebble layer with new log level."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "log-level": "debug"},
        containers={container},
        leader=True,
    )
    state_out = ctx.run(ctx.on.config_changed(), state_in)
    expected_plan = {
        "services": {
            "hardware-api": {
                "override": "replace",
                "summary": "Hardware API server",
                "command": "uvicorn hwapi.main:app --host 0.0.0.0 --port 30000 --log-level debug",
                "startup": "enabled",
                # Since environment is empty, Layer.to_dict() omits it.
            }
        }
    }
    assert state_out.get_container(container.name).plan == expected_plan
    assert state_out.unit_status == testing.ActiveStatus()
    assert (
        state_out.get_container(container.name).service_statuses["hardware-api"]
        == ops.pebble.ServiceStatus.ACTIVE
    )


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed_not_leader(
    patched_submit_to_traefik: MagicMock, ingress_template: str
):
    """Tests that route_relation_changed does nothing if not leader."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    relation = testing.Relation("traefik-route")
    state_in = testing.State(
        config={"port": 30000, "log-level": "info"},
        containers={container},
        relations={relation},
        leader=False,
    )
    with patch("builtins.open", mock_open(read_data=ingress_template)):
        ctx.run(ctx.on.relation_changed(relation), state_in)
    patched_submit_to_traefik.assert_not_called()


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed_no_external_hostname(patched_submit_to_traefik: MagicMock):
    """Tests the route relation changed event blocks with no hostname."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    relation = testing.Relation("traefik-route")
    state_in = testing.State(
        config={"port": 30000, "log-level": "debug"},
        containers={container},
        relations={relation},
        leader=True,
    )
    state_out = ctx.run(ctx.on.relation_changed(relation), state_in)
    assert isinstance(state_out.unit_status, testing.BlockedStatus)
    assert state_out.unit_status.message == "external_hostname must be set on traefik-k8s"
    patched_submit_to_traefik.assert_not_called()


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed_not_ready(
    patched_submit_to_traefik: MagicMock, ingress_template: str
):
    """Tests the route relation changed event is deferred when not ready."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    remote_app_data = {"external_host": "hw.local"}
    relation = testing.Relation("traefik-route", remote_app_data=remote_app_data)
    state_in = testing.State(
        config={"port": 30000, "log-level": "debug"},
        containers={container},
        relations={relation},
        leader=True,
    )
    with (
        patch("charm.TraefikRouteRequirer.is_ready", MagicMock(return_value=False)),
        patch("builtins.open", mock_open(read_data=ingress_template)),
    ):
        ctx.run(ctx.on.relation_changed(relation), state_in)
    patched_submit_to_traefik.assert_not_called()


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed(patched_submit_to_traefik: MagicMock, ingress_template: str):
    """Tests the route relation changed event on success."""
    ctx = testing.Context(HardwareApiCharm)
    container = testing.Container(name="hardware-api", can_connect=True)
    remote_app_data = {"external_host": "hw.local"}
    relation = testing.Relation("traefik-route", remote_app_data=remote_app_data)
    state_in = testing.State(
        config={"port": 30000, "log-level": "debug"},
        containers={container},
        relations={relation},
        leader=True,
    )
    with patch("builtins.open", mock_open(read_data=ingress_template)):
        ctx.run(ctx.on.relation_changed(relation), state_in)
    patched_submit_to_traefik.assert_called_once()
