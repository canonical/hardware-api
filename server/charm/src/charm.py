#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
#
# Learn more at: https://juju.is/docs/sdk
"""Hardware API Charm."""

import logging
import shlex

import ops
from charms.nginx_ingress_integrator.v0.nginx_route import require_nginx_route

# Log messages can be retrieved using juju debug-log
logger = logging.getLogger(__name__)

VALID_LOG_LEVELS = ["info", "debug", "warning", "error", "critical"]


class HardwareApiCharm(ops.CharmBase):
    """Charm the service."""

    def __init__(self, *args):
        super().__init__(*args)
        self._setup_nginx()
        self.framework.observe(
            self.on["hardware-api"].pebble_ready,
            self._on_hardware_api_pebble_ready,
        )
        self.framework.observe(self.on.config_changed, self._on_config_changed)

    def _setup_nginx(self):
        require_nginx_route(
            charm=self,
            service_hostname=str(self.config["hostname"]),
            service_name=self.app.name,
            service_port=int(self.config["port"]),
        )

    def _on_hardware_api_pebble_ready(self, event: ops.PebbleReadyEvent):
        container = event.workload
        container.add_layer("hardware-api", self._pebble_layer, combine=True)
        container.replan()
        self.unit.status = ops.ActiveStatus()

    def _on_config_changed(self, event: ops.ConfigChangedEvent):
        log_level = str(self.model.config["log-level"]).lower()
        if log_level not in VALID_LOG_LEVELS:
            self.unit.status = ops.BlockedStatus(f"invalid log level: '{log_level}'")
            return

        container = self.unit.get_container("hardware-api")
        if not container.can_connect():
            self.unit.status = ops.WaitingStatus("waiting for Pebble API")
            event.defer()
            return

        container.add_layer("hardware-api", self._pebble_layer, combine=True)
        container.replan()
        logger.debug("Log level changed to '%s'", log_level)
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
                            f"{self.config['port']}",
                            "--log-level",
                            f"{self.model.config['log-level']}",
                        ]
                    ),
                    "startup": "enabled",
                    "environment": self._app_environment,
                }
            },
        }


if __name__ == "__main__":  # pragma: nocover
    ops.main(HardwareApiCharm)  # type: ignore
