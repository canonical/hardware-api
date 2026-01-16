#!/usr/bin/env python3
# Copyright 2024 Canonical Ltd
# See LICENSE file for licensing details.
"""Integration tests for the Hardware API Charm."""

import logging
from pathlib import Path

import jubilant
import requests
import yaml

from .helpers import DNSResolverHTTPAdapter, app_is_up, retry

logger = logging.getLogger(__name__)

METADATA = yaml.safe_load(Path("charmcraft.yaml").read_text(encoding="utf-8"))
APP_NAME = METADATA["name"]
PORT = 80
INGRESS_NAME = "ingress"
EXTERNAL_HOSTNAME = "hw.internal"


def test_deploy(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test."""
    upstream_source = METADATA["resources"]["hardware-api-image"]["upstream-source"]
    resources = {"hardware-api-image": upstream_source}
    config = {"port": PORT}
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources, config=config)
    juju.wait(jubilant.all_active)


@retry(retry_num=5, retry_sleep_sec=5)
def test_application_is_up(juju: jubilant.Juju):
    """Test that the application is up and running."""
    ip = juju.status().apps[APP_NAME].units[f"{APP_NAME}/0"].address
    base_url = f"http://{ip}:{PORT}"
    assert app_is_up(base_url)


def test_deploy_ingress(juju: jubilant.Juju):
    """Deploy the ingress charm."""
    config = {"external_hostname": EXTERNAL_HOSTNAME}
    juju.deploy(
        "traefik-k8s", channel="latest/stable", app=INGRESS_NAME, config=config, trust=True
    )
    juju.integrate(f"{APP_NAME}:traefik-route", f"{INGRESS_NAME}:traefik-route")
    juju.wait(jubilant.all_active, timeout=600)


@retry(retry_num=5, retry_sleep_sec=5)
def test_ingress_is_up(juju: jubilant.Juju):
    """Test that the application is reachable via the ingress."""
    ingress_ip = juju.status().apps[INGRESS_NAME].units[f"{INGRESS_NAME}/0"].address
    session = requests.Session()
    session.mount("http://", DNSResolverHTTPAdapter(EXTERNAL_HOSTNAME, ingress_ip))
    session.mount("https://", DNSResolverHTTPAdapter(EXTERNAL_HOSTNAME, ingress_ip))
    base_url = f"http://{EXTERNAL_HOSTNAME}/"
    assert app_is_up(base_url, session=session)
