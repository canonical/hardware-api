#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
#
# Learn more at: https://juju.is/docs/sdk
"""Hardware API Charm."""

import logging
import shlex

import ops
from charms.data_platform_libs.v0.data_interfaces import (
    DatabaseCreatedEvent,
    DatabaseEndpointsChangedEvent,
    DatabaseRequires,
)
from charms.nginx_ingress_integrator.v0.nginx_route import require_nginx_route
from charms.traefik_k8s.v2.ingress import (
    IngressPerAppReadyEvent,
    IngressPerAppRequirer,
    IngressPerAppRevokedEvent,
)

from config import HardwareApiConfig

logger = logging.getLogger(__name__)

CONTAINER_NAME = "hardware-api"
LAYER_LABEL = "hardware-api"
SERVICE_NAME = "hardware-api"


class HardwareApiCharm(ops.CharmBase):
    """Hardware API Charm."""

    def __init__(self, *args):
        super().__init__(*args)
        self.container = self.unit.get_container(CONTAINER_NAME)
        self.typed_config = self.load_config(HardwareApiConfig, errors="blocked")
        self._setup_nginx()
        self._setup_ingress()
        self._setup_db()
        self.framework.observe(
            self.on[CONTAINER_NAME].pebble_ready,
            self._on_hardware_api_pebble_ready,
        )
        self.framework.observe(self.on.config_changed, self._on_config_changed)

    def _setup_nginx(self):
        """Set up the NGINX requirer."""
        # TODO: Remove once NGINX support is dropped
        require_nginx_route(
            charm=self,
            service_hostname=self.typed_config.hostname,
            service_name=self.app.name,
            service_port=self.typed_config.port,
        )
        self.framework.observe(
            self.on.nginx_route_relation_broken, self._on_route_relation_changed
        )
        self.framework.observe(
            self.on.nginx_route_relation_changed, self._on_route_relation_changed
        )

    def _setup_ingress(self):
        """Set up the Ingress requirer."""
        self.ingress = IngressPerAppRequirer(self, "ingress", port=self.typed_config.port)
        self.framework.observe(self.on.ingress_relation_changed, self._on_route_relation_changed)
        self.framework.observe(self.on.ingress_relation_broken, self._on_route_relation_changed)
        self.framework.observe(self.ingress.on.ready, self._on_ingress_ready)
        self.framework.observe(self.ingress.on.revoked, self._on_ingress_revoked)

    def _setup_db(self):
        """Set up the database requirer."""
        self.database = DatabaseRequires(self, relation_name="database", database_name="hwapi")
        self.framework.observe(self.database.on.database_created, self._on_database_endpoint)
        self.framework.observe(self.database.on.endpoints_changed, self._on_database_endpoint)

    def _on_hardware_api_pebble_ready(self, event: ops.PebbleReadyEvent):
        self.replan()

    def _on_config_changed(self, event: ops.ConfigChangedEvent):
        if not self.container.can_connect():
            self.unit.status = ops.WaitingStatus("waiting for Pebble API")
            event.defer()
            return
        self.replan()
        logger.debug("Log level changed to '%s'", self.typed_config.log_level)

    def _on_route_relation_changed(self, event: ops.RelationEvent):
        """Handle a route relation change event."""
        if len(self.get_active_route_providers()) > 1:
            self.unit.status = ops.BlockedStatus("multiple route providers related")
            return
        self.unit.status = ops.ActiveStatus()

    def _on_ingress_ready(self, event: IngressPerAppReadyEvent):
        """Handle the Ingress ready event."""
        if len(self.get_active_route_providers()) > 1:
            self.unit.status = ops.BlockedStatus("multiple route providers related")
            return
        logger.info("Ingress is ready with URL: %s", event.url)

    def _on_ingress_revoked(self, event: IngressPerAppRevokedEvent):
        """Handle the Ingress revoked event."""
        logger.info("Ingress revoked")

    def get_active_route_providers(self):
        """Return a list of active route providers."""
        providers = []
        if self.model.get_relation("nginx-route"):
            providers.append("nginx-route")
        if self.model.get_relation("ingress"):
            providers.append("ingress")
        return providers

    def replan(self):
        """Replan the Pebble layer."""
        self.unit.status = ops.MaintenanceStatus("configuring Pebble layer")
        self.container.add_layer(LAYER_LABEL, self._pebble_layer, combine=True)
        self.container.replan()
        self.unit.status = ops.ActiveStatus()

    def _on_database_endpoint(self, event: DatabaseCreatedEvent | DatabaseEndpointsChangedEvent):
        """Handle a database endpoint event."""
        self.replan()

    @property
    def db_url(self) -> str:
        """Database URL."""
        relations = self.database.fetch_relation_data()
        for data in relations.values():
            if not data:
                continue
            return f"postgresql://{data['username']}:{data['password']}@{data['endpoints']}/hwapi"
        logger.error("Database URL not available yet")
        self.unit.status = ops.WaitingStatus("waiting for database relation")
        raise SystemExit(0)

    @property
    def _app_environment(self):
        """Environment variables needed by the application."""
        env = {
            "DB_URL": self.db_url,
        }
        return env

    @property
    def _pebble_layer(self) -> ops.pebble.LayerDict:
        return {
            "summary": "Hardware API",
            "description": "pebble config layer for hardware-api",
            "services": {
                SERVICE_NAME: {
                    "override": "replace",
                    "summary": "Hardware API server",
                    "command": shlex.join(
                        [
                            "uvicorn",
                            "hwapi.main:app",
                            "--host",
                            "0.0.0.0",
                            "--port",
                            f"{self.typed_config.port}",
                            "--log-level",
                            f"{self.typed_config.log_level}",
                        ]
                    ),
                    "startup": "enabled",
                    "environment": self._app_environment,
                }
            },
        }


if __name__ == "__main__":  # pragma: nocover
    ops.main(HardwareApiCharm)
