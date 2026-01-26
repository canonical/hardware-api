#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
#
# Learn more at: https://juju.is/docs/sdk
"""Hardware API Charm."""

import logging
import shlex
from typing import Literal

import ops
import pydantic
from charms.nginx_ingress_integrator.v0.nginx_route import require_nginx_route
from charms.traefik_k8s.v0.traefik_route import TraefikRouteRequirer

# Log messages can be retrieved using juju debug-log
logger = logging.getLogger(__name__)


class HardwareApiConfig(pydantic.BaseModel):
    """Hardware API Charm configuration."""

    log_level: Literal["info", "debug", "warning", "error", "critical"] = "info"
    port: int = 30000
    hostname: str = "hw"


class HardwareApiCharm(ops.CharmBase):
    """Hardware API Charm."""

    def __init__(self, *args):
        super().__init__(*args)
        self.typed_config = self.load_config(HardwareApiConfig, errors="blocked")
        self._setup_nginx()
        self._setup_traefik()
        self.framework.observe(
            self.on["hardware-api"].pebble_ready,
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

    def _setup_traefik(self):
        """Set up the Traefik requirer."""
        self.traefik = TraefikRouteRequirer(
            self,
            self.model.get_relation("traefik-route"),  # type: ignore
            "traefik-route",
        )
        self.framework.observe(self.traefik.on.ready, self._on_route_relation_changed)

    def _on_hardware_api_pebble_ready(self, event: ops.PebbleReadyEvent):
        container = event.workload
        container.add_layer("hardware-api", self._pebble_layer, combine=True)
        container.replan()
        self.unit.status = ops.ActiveStatus()

    def _on_config_changed(self, event: ops.ConfigChangedEvent):
        container = self.unit.get_container("hardware-api")
        if not container.can_connect():
            self.unit.status = ops.WaitingStatus("waiting for Pebble API")
            event.defer()
            return

        container.add_layer("hardware-api", self._pebble_layer, combine=True)
        container.replan()
        logger.debug("Log level changed to '%s'", self.typed_config.log_level)
        self.unit.status = ops.ActiveStatus()

    def _on_route_relation_changed(self, event: ops.RelationEvent):
        """Handle a route relation change event."""
        nginx_route = self.model.get_relation("nginx-route")
        traefik_route = self.model.get_relation("traefik-route")
        if nginx_route and traefik_route:
            # TODO: Remove once NGINX support is dropped
            self.unit.status = ops.BlockedStatus("multiple route providers related")
            return
        if not self.unit.is_leader():
            return

        if not self.traefik.is_ready():
            logger.info("traefik-route relation not ready yet")
            return

        self.unit.open_port("tcp", self.typed_config.port)
        self.unit.status = ops.MaintenanceStatus("configuring route")
        service_url = (
            f"http://{self.app.name}.{self.model.name}.svc.cluster.local:{self.typed_config.port}"
        )
        # HACK: Fallback to internal domain on initial setup
        external_hostname = self.traefik.external_host or service_url
        identifier = f"{self.model.name}-{self.app.name}"
        config = {
            "http": {
                "middlewares": {
                    "https-redirect": {"redirectScheme": {"scheme": "https", "permanent": True}}
                },
                "routers": {
                    f"juju-{identifier}-router": {
                        "rule": f"Host(`{external_hostname}`)",
                        "service": f"juju-{identifier}-service",
                        "entryPoints": ["web"],
                        "middlewares": ["https-redirect"],
                    },
                    f"juju-{identifier}-router-tls": {
                        "rule": f"Host(`{external_hostname}`)",
                        "service": f"juju-{identifier}-service",
                        "entryPoints": ["websecure"],
                        "tls": {},
                    },
                },
                "services": {
                    f"juju-{identifier}-service": {
                        "loadBalancer": {"servers": [{"url": service_url}], "passHostHeader": True}
                    }
                },
            }
        }
        logger.info("submitting traefik config for %s", external_hostname)
        self.traefik.submit_to_traefik(config=config)
        self.unit.status = ops.ActiveStatus()

    @property
    def _app_environment(self):
        """Environment variables needed by the application."""
        env = {}
        return env

    @property
    def _pebble_layer(self) -> ops.pebble.LayerDict:
        return {
            "summary": "Hardware API",
            "description": "pebble config layer for hardware-api",
            "services": {
                "hardware-api": {
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
    ops.main(HardwareApiCharm)  # type: ignore
