#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
#
# Learn more at: https://juju.is/docs/sdk

"""Charm the service.

Refer to the following tutorial that will help you
develop a new k8s charm using the Operator Framework:

https://juju.is/docs/sdk/create-a-minimal-kubernetes-charm
"""

import logging

import ops

# Log messages can be retrieved using juju debug-log
logger = logging.getLogger(__name__)

VALID_LOG_LEVELS = ["info", "debug", "warning", "error", "critical"]


class CharmCharm(ops.CharmBase):
    """Charm the service."""

    def __init__(self, *args):
        super().__init__(*args)
        self.framework.observe(self.on['hardware_api'].pebble_ready, self._on_hardware_api_pebble_ready)
        self.framework.observe(self.on.config_changed, self._on_config_changed)

    def _on_hardware_api_pebble_ready(self, event: ops.PebbleReadyEvent):
        """Define and start a workload using the Pebble API.

        Change this example to suit your needs. You'll need to specify the right entrypoint and
        environment configuration for your specific workload.

        Learn more about interacting with Pebble at at https://juju.is/docs/sdk/pebble.
        """
        container = event.workload
        container.add_layer("hardware_api", self._pebble_layer, combine=True)
        container.replan()
        self.unit.status = ops.ActiveStatus()

    def _on_config_changed(self, event: ops.ConfigChangedEvent):
        """Handle changed configuration.

        Change this example to suit your needs. If you don't need to handle config, you can remove
        this method.

        Learn more about config at https://juju.is/docs/sdk/config
        """
        log_level = self.model.config["log-level"].lower()

        if log_level in VALID_LOG_LEVELS:
            container = self.unit.get_container("hardware_api")
            if container.can_connect():
                container.add_layer("hardware_api", self._pebble_layer, combine=True)
                container.replan()
                logger.debug("Log level changed to '%s'", log_level)
                self.unit.status = ops.ActiveStatus()
            else:
                event.defer()
                self.unit.status = ops.WaitingStatus("waiting for Pebble API")
        else:
            self.unit.status = ops.BlockedStatus("invalid log level: '{log_level}'")

    @property
    def _pebble_layer(self) -> ops.pebble.LayerDict:
        """Return a dictionary representing a Pebble layer."""
        return {
            "summary": "httpbin layer",
            "description": "pebble config layer for hardware_api",
            "services": {
                "hardware_api": {
                    "override": "replace",
                    "summary": "test observer API server",
                    "command": " ".join(
                        [
                            "uvicorn",
                            "hwapi.main:app",
                            "--host",
                            "0.0.0.0",
                            f"--port={self.config['port']}",
                            "--log-level",
                            self.model.config['log-level']
                        ]
                    ),
                    "startup": "enabled",
                    "environment": self._app_environment,
                }
            },
        }


if __name__ == "__main__":  # pragma: nocover
    ops.main(CharmCharm)  # type: ignore
