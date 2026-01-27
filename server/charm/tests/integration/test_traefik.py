#!/usr/bin/env python3
# Copyright (C) 2026 Canonical Ltd.
# See LICENSE file for licensing details.
"""Integration tests for the Hardware API Charm behind Traefik."""

import logging
from pathlib import Path

import jubilant
import requests
import yaml

from .helpers import DNSResolverHTTPAdapter, app_is_up, get_k8s_ingress_ip, retry

logger = logging.getLogger(__name__)

METADATA = yaml.safe_load(Path("charmcraft.yaml").read_text(encoding="utf-8"))
APP_NAME = METADATA["name"]
PORT = 80
INGRESS_NAME = "ingress"
EXTERNAL_HOSTNAME = "hw.internal"


def test_deploy_ingress(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test and the ingress charm."""
    upstream_source = METADATA["resources"]["hardware-api-image"]["upstream-source"]
    resources = {"hardware-api-image": upstream_source}
    config = {"port": PORT}
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources, config=config)

    config = {"external_hostname": EXTERNAL_HOSTNAME}
    juju.deploy(
        "traefik-k8s", channel="latest/stable", app=INGRESS_NAME, config=config, trust=True
    )
    juju.integrate(f"{APP_NAME}:traefik-route", f"{INGRESS_NAME}:traefik-route")
    juju.wait(jubilant.all_active)


@retry(retry_num=5, retry_sleep_sec=5)
def test_ingress_is_up(juju: jubilant.Juju):
    """Test that the application is reachable via the ingress."""
    model = juju.show_model()
    ingress_ip = get_k8s_ingress_ip(model.short_name, f"{INGRESS_NAME}-lb")
    session = requests.Session()
    dns_resolver = DNSResolverHTTPAdapter(EXTERNAL_HOSTNAME, ingress_ip)
    session.mount("http://", dns_resolver)
    session.mount("https://", dns_resolver)
    base_url = f"http://{EXTERNAL_HOSTNAME}"
    assert app_is_up(base_url, session=session)
