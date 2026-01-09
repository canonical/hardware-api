#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
"""Integration tests for the Hardware API Charm."""

import logging
from pathlib import Path

import jubilant
import requests
import yaml

from .helpers import all_active_idle, get_ingress_endpoint

logger = logging.getLogger(__name__)

METADATA = yaml.safe_load(Path("charmcraft.yaml").read_text(encoding="utf-8"))
APP_NAME = METADATA["name"]
INGRESS_NAME = "ingress"


def test_deploy(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test."""
    resources = {
        "hardware-api-image": METADATA["resources"]["hardware-api-image"]["upstream-source"],
    }
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources)
    juju.wait(jubilant.all_active)


def test_relate_ingress(juju: jubilant.Juju):
    """Relate the charm under test to the traefik operator."""
    juju.deploy("traefik-k8s", channel="latest/stable", app=INGRESS_NAME, trust=True)
    juju.integrate(f"{APP_NAME}:ingress", f"{INGRESS_NAME}:ingress")
    juju.wait(all_active_idle)

    endpoint = get_ingress_endpoint(juju, INGRESS_NAME, APP_NAME)
    url = f"{endpoint}/"
    logger.info("Querying endpoint: %s", url)
    response = requests.get(url, timeout=15, verify=False)
    assert response.ok
    assert response.text == "Hardware Information API (hwapi) server"
