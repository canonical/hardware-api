#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
"""Integration tests for the Hardware API Charm."""

import logging
from pathlib import Path

import jubilant
import yaml

logger = logging.getLogger(__name__)

METADATA = yaml.safe_load(Path("charmcraft.yaml").read_text(encoding="utf-8"))
APP_NAME = METADATA["name"]
PORT = 80


def test_deploy(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test."""
    upstream_source = METADATA["resources"]["hardware-api-image"]["upstream-source"]
    resources = {"hardware-api-image": upstream_source}
    config = {"port": PORT}
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources, config=config)
    juju.wait(jubilant.all_active)


def test_relate_ingress(juju: jubilant.Juju):
    """Relate the charm under test to the nginx ingress integrator."""
    juju.deploy("nginx-ingress-integrator", channel="latest/stable", trust=True)
    juju.integrate(f"{APP_NAME}:nginx-route", "nginx-ingress-integrator:nginx-route")
    juju.wait(jubilant.all_active)
