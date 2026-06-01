# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
#
# Learn more about testing at: https://juju.is/docs/sdk/testing
"""Unit tests for Hardware API Charm."""

import dataclasses
import logging
from unittest.mock import patch

import ops
import pytest
from ops import testing

logger = logging.getLogger(__name__)


def test_pebble_layer(ctx: testing.Context, container: testing.Container):
    """Tests that the Pebble layer is correctly set up."""
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


def test_config_changed_invalid_log_level(ctx: testing.Context, container: testing.Container):
    """Tests that an invalid log level in config_changed sets BlockedStatus."""
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "invalid"},
        containers={container},
        leader=True,
    )
    with pytest.raises(testing.errors.UncaughtCharmError):
        state_out = ctx.run(ctx.on.config_changed(), state_in)
        assert isinstance(state_out.unit_status, testing.BlockedStatus)


def test_config_changed_pebble_not_ready(ctx: testing.Context, container: testing.Container):
    """Tests that config_changed defers event if Pebble is not ready."""
    container = dataclasses.replace(container, can_connect=False)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "info"},
        containers={container},
        leader=True,
    )
    with patch("ops.framework.EventBase.defer") as patched_defer:
        ctx.run(ctx.on.config_changed(), state_in)
    patched_defer.assert_called_once()


def test_config_changed_updates_pebble_layer(ctx: testing.Context, container: testing.Container):
    """Tests that config_changed updates the Pebble layer with new log level."""
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


def test_blocked_status_on_multiple_ingress_providers(
    ctx: testing.Context, container: testing.Container, ingress_relation: testing.Relation
):
    """Tests that status is blocked when both ingress providers are connected."""
    # TODO: Remove once NGINX support is dropped
    nginx_relation = testing.Relation("nginx-route")
    state_in = testing.State(
        containers={container},
        leader=True,
        relations={ingress_relation, nginx_relation},
    )
    state_out = ctx.run(ctx.on.relation_changed(ingress_relation), state_in)
    assert isinstance(state_out.unit_status, testing.BlockedStatus)


def test_ingress_relation_changed(
    ctx: testing.Context, container: testing.Container, ingress_relation: testing.Relation
):
    """Tests the ingress relation changed event on success."""
    state_in = testing.State(containers={container}, relations={ingress_relation}, leader=True)
    state_out = ctx.run(ctx.on.relation_changed(ingress_relation), state_in)
    assert isinstance(state_out.unit_status, testing.ActiveStatus)


def test_ingress_relation_broken(
    ctx: testing.Context, container: testing.Container, ingress_relation: testing.Relation
):
    """Tests the ingress relation broken event on success."""
    state_in = testing.State(containers={container}, relations={ingress_relation}, leader=True)
    state_out = ctx.run(ctx.on.relation_broken(ingress_relation), state_in)
    assert isinstance(state_out.unit_status, testing.ActiveStatus)
