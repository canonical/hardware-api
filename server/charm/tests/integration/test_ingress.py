#!/usr/bin/env python3
# Copyright (C) 2026 Canonical Ltd.
# See LICENSE file for licensing details.
"""Integration tests for the Hardware API Charm behind Ingress Configurator."""

import logging
from pathlib import Path

import jubilant
import pytest
import yaml

logger = logging.getLogger(__name__)

METADATA = yaml.safe_load(Path("charmcraft.yaml").read_text(encoding="utf-8"))
APP_NAME = METADATA["name"]
PORT = 80
INGRESS_NAME = "ingress"
EXTERNAL_HOSTNAME = "hw.internal"


@pytest.mark.juju_setup
def test_deploy(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test."""
    upstream_source = METADATA["resources"]["hardware-api-image"]["upstream-source"]
    resources = {"hardware-api-image": upstream_source}
    config = {"port": PORT}
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources, config=config)
    juju.wait(jubilant.all_active)


@pytest.mark.juju_setup
def test_deploy_ingress(juju: jubilant.Juju):
    """Deploy the ingress charm."""
    juju.deploy(
        "ingress-configurator",
        channel="latest/edge",
        app=INGRESS_NAME,
        config={"hostname": EXTERNAL_HOSTNAME},
        trust=True,
    )
    juju.integrate(f"{APP_NAME}:ingress", INGRESS_NAME)
    juju.wait(jubilant.all_active)


@pytest.mark.juju_teardown
def test_destroy(juju: jubilant.Juju):
    """Tear down the charm under test."""
    juju.remove_application(INGRESS_NAME)
    juju.remove_application(APP_NAME)
