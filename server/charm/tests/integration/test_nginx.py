#!/usr/bin/env python3
# Copyright (C) 2026 Canonical Ltd.
# See LICENSE file for licensing details.
# TODO: Remove once NGINX support is dropped
"""Integration tests for the Hardware API Charm behind NGINX."""

import logging
import re
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


def test_deploy_ingress(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test and the ingress charm."""
    upstream_source = METADATA["resources"]["hardware-api-image"]["upstream-source"]
    resources = {"hardware-api-image": upstream_source}
    config = {"port": PORT, "hostname": EXTERNAL_HOSTNAME}
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources, config=config)

    juju.deploy("nginx-ingress-integrator", channel="latest/stable", app=INGRESS_NAME, trust=True)
    juju.integrate(f"{APP_NAME}:nginx-route", f"{INGRESS_NAME}:nginx-route")
    juju.wait(jubilant.all_active)


@retry(retry_num=5, retry_sleep_sec=5)
def test_ingress_is_up(juju: jubilant.Juju):
    """Test that the application is reachable via the ingress."""
    status_message = juju.status().apps[INGRESS_NAME].app_status.message
    match = re.search(r"Ingress IP\(s\): ([\d.]+)", status_message)
    assert match, f"Could not find ingress IP in status: {status_message}"
    ingress_ip = match.group(1)
    session = requests.Session()
    dns_resolver = DNSResolverHTTPAdapter(EXTERNAL_HOSTNAME, ingress_ip)
    session.mount("http://", dns_resolver)
    session.mount("https://", dns_resolver)
    base_url = f"http://{EXTERNAL_HOSTNAME}"
    assert app_is_up(base_url, session=session)
