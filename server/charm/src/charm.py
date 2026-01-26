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
        self.framework.observe(
            self.on["hardware-api"].pebble_ready,
            self._on_hardware_api_pebble_ready,
        )
        self.framework.observe(self.on.config_changed, self._on_config_changed)

    def _setup_nginx(self):
        require_nginx_route(
            charm=self,
            service_hostname=self.typed_config.hostname,
            service_name=self.app.name,
            service_port=self.typed_config.port,
        )

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
