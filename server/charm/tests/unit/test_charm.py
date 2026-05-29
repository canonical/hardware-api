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

from tests.unit.conftest import DatabaseFixture

logger = logging.getLogger(__name__)


def test_pebble_ready_waits_for_database(ctx: testing.Context, container: testing.Container):
    """Tests that pebble_ready waits for the database relation before starting."""
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "info"},
        containers={container},
        leader=True,
    )
    with ctx(ctx.on.pebble_ready(container), state_in) as mgr:
        with pytest.raises(SystemExit):
            mgr.run()
        assert mgr.charm.unit.status == ops.WaitingStatus("waiting for database relation")


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


def test_config_changed_waits_for_database(ctx: testing.Context, container: testing.Container):
    """Tests that config_changed waits for the database relation before replanning."""
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "debug"},
        containers={container},
        leader=True,
    )
    with ctx(ctx.on.config_changed(), state_in) as mgr:
        with pytest.raises(SystemExit):
            mgr.run()
        assert mgr.charm.unit.status == ops.WaitingStatus("waiting for database relation")


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


def test_database_relation_sets_db_url(ctx: testing.Context, database: DatabaseFixture):
    """Tests that credentials from the database relation build the database URL."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(containers={container}, relations={database.relation}, leader=True)
    with ctx(ctx.on.relation_changed(database.relation, remote_unit=0), state_in) as mgr:
        mgr.run()
        assert mgr.charm.db_url == database.url


def test_database_relation_builds_pebble_layer(ctx: testing.Context, database: DatabaseFixture):
    """Tests that the database URL is injected into the Pebble layer once available."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "info"},
        containers={container},
        relations={database.relation},
        leader=True,
    )
    with ctx(ctx.on.relation_changed(database.relation, remote_unit=0), state_in) as mgr:
        mgr.run()
        assert mgr.charm._app_environment == {"DB_URL": database.url}
        assert mgr.charm._pebble_layer["services"]["hardware-api"] == {
            "override": "replace",
            "summary": "Hardware API server",
            "command": "uvicorn hwapi.main:app --host 0.0.0.0 --port 30000 --log-level info",
            "startup": "enabled",
            "environment": {"DB_URL": database.url},
        }


def test_pebble_layer_reflects_log_level(ctx: testing.Context, database: DatabaseFixture):
    """Tests that the configured log level is propagated to the Pebble command."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(
        config={"port": 30000, "hostname": "hw", "log-level": "debug"},
        containers={container},
        relations={database.relation},
        leader=True,
    )
    with ctx(ctx.on.relation_changed(database.relation, remote_unit=0), state_in) as mgr:
        mgr.run()
        command = mgr.charm._pebble_layer["services"]["hardware-api"]["command"]
        assert command == "uvicorn hwapi.main:app --host 0.0.0.0 --port 30000 --log-level debug"


def test_database_relation_without_credentials_does_not_set_db_url(ctx: testing.Context):
    """Tests that a relation without provider credentials does not emit resource_created."""
    container = testing.Container(name="hardware-api", can_connect=True)
    relation = testing.Relation(
        "database",
        interface="postgresql_client",
        remote_app_name="postgresql",
        remote_units_data={0: {}},
    )
    state_in = testing.State(containers={container}, relations={relation}, leader=True)
    with ctx(ctx.on.relation_changed(relation, remote_unit=0), state_in) as mgr:
        mgr.run()
        with pytest.raises(SystemExit):
            mgr.charm.db_url


def test_db_url_waiting_status_without_relation(ctx: testing.Context):
    """Tests that db_url blocks with a waiting status when no database is related."""
    container = testing.Container(name="hardware-api", can_connect=True)
    state_in = testing.State(containers={container}, leader=True)
    with ctx(ctx.on.update_status(), state_in) as mgr:
        with pytest.raises(SystemExit):
            _ = mgr.charm.db_url
        assert mgr.charm.unit.status == ops.WaitingStatus("waiting for database relation")
