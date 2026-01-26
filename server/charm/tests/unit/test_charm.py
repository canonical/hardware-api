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

logger = logging.getLogger(__name__)


def test_pebble_layer(ctx: testing.Context):
    """Tests that the Pebble layer is correctly set up."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "info"},
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


def test_config_changed_invalid_log_level(ctx: testing.Context):
    """Tests that an invalid log level in config_changed sets BlockedStatus."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "invalid"},
        containers={container},
        leader=True,
    )
    with pytest.raises(testing.errors.UncaughtCharmError):
        state_out = ctx.run(ctx.on.config_changed(), state_in)
        assert isinstance(state_out.unit_status, testing.BlockedStatus)


def test_config_changed_pebble_not_ready(ctx: testing.Context):
    """Tests that config_changed defers event if Pebble is not ready."""
    container = testing.Container(name="hardware-api", can_connect=False)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "info"},
        containers={container},
        leader=True,
    )
    with patch("ops.framework.EventBase.defer") as patched_defer:
        ctx.run(ctx.on.config_changed(), state_in)
    patched_defer.assert_called_once()


def test_config_changed_updates_pebble_layer(ctx: testing.Context):
    """Tests that config_changed updates the Pebble layer with new log level."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "debug"},
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


def test_blocked_status_on_multiple_ingress_providers(ctx: testing.Context):
    """Tests that status is blocked when both ingress providers are connected."""
    # TODO: Remove once NGINX support is dropped
    container = testing.Container(name="hardware-api", can_connect=True)
    traefik_relation = testing.Relation("traefik-route")
    nginx_relation = testing.Relation("nginx-route")
    state_in = testing.State(
        containers={container},
        leader=True,
        relations={traefik_relation, nginx_relation},
    )
    state_out = ctx.run(ctx.on.relation_changed(traefik_relation), state_in)
    assert isinstance(state_out.unit_status, testing.BlockedStatus)


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed_not_leader(
    patched_submit_to_traefik: MagicMock, ctx: testing.Context, ingress_template: str
):
    """Tests that non-leader units do not submit Traefik config."""
    container = testing.Container(name="hardware-api", can_connect=True)
    remote_app_data = {"external_host": "hw.local"}
    relation = testing.Relation("traefik-route", remote_app_data=remote_app_data)
    state_in = testing.State(containers={container}, relations={relation}, leader=False)
    with patch("builtins.open", mock_open(read_data=ingress_template)):
        ctx.run(ctx.on.relation_changed(relation), state_in)
    patched_submit_to_traefik.assert_not_called()


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed_not_ready(
    patched_submit_to_traefik: MagicMock, ctx: testing.Context, ingress_template: str
):
    """Tests the route relation changed event is deferred when not ready."""
    container = testing.Container(name="hardware-api", can_connect=True)
    remote_app_data = {"external_host": "hw.local"}
    relation = testing.Relation("traefik-route", remote_app_data=remote_app_data)
    state_in = testing.State(containers={container}, relations={relation}, leader=True)
    with (
        patch("charm.TraefikRouteRequirer.is_ready", MagicMock(return_value=False)),
        patch("builtins.open", mock_open(read_data=ingress_template)),
    ):
        ctx.run(ctx.on.relation_changed(relation), state_in)
    patched_submit_to_traefik.assert_not_called()


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_changed(
    patched_submit_to_traefik: MagicMock, ctx: testing.Context, ingress_template: str
):
    """Tests the route relation changed event on success."""
    container = testing.Container(name="hardware-api", can_connect=True)
    remote_app_data = {"external_host": "hw.local"}
    relation = testing.Relation("traefik-route", remote_app_data=remote_app_data)
    state_in = testing.State(containers={container}, relations={relation}, leader=True)
    with (
        patch("builtins.open", mock_open(read_data=ingress_template)),
        ctx(ctx.on.relation_changed(relation), state_in) as mgr,
    ):
        assert mgr.charm.traefik.is_ready()
        assert mgr.charm.traefik._relation is not None
    patched_submit_to_traefik.assert_called_once()


@patch("charm.TraefikRouteRequirer.submit_to_traefik")
def test_route_relation_broken(patched_submit_to_traefik: MagicMock, ctx: testing.Context):
    """Tests the route relation changed event on success."""
    # Test relation broken
    container = testing.Container(name="hardware-api", can_connect=True)
    remote_app_data = {"external_host": "hw.local"}
    relation = testing.Relation("traefik-route", remote_app_data=remote_app_data)
    state_in = testing.State(containers={container}, relations={relation}, leader=True)
    ctx.run(ctx.on.relation_broken(relation), state_in)
    patched_submit_to_traefik.assert_not_called()
    # Test relation status after removal
    state_in = testing.State(containers={container}, leader=True)
    with ctx(ctx.on.update_status(), state_in) as mgr:
        assert not mgr.charm.traefik.is_ready()
        assert mgr.charm.traefik._relation is None
    patched_submit_to_traefik.assert_not_called()
