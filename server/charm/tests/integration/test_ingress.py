#!/usr/bin/env python3
# Copyright (C) 2026 Canonical Ltd.
# See LICENSE file for licensing details.
"""Integration tests for the Hardware API Charm behind Ingress Configurator."""

import logging
from pathlib import Path

import jubilant
import pytest
import requests
import yaml

from .helpers import DNSResolverHTTPAdapter, app_is_up, model_short_name, retry

logger = logging.getLogger(__name__)

METADATA = yaml.safe_load(Path("charmcraft.yaml").read_text(encoding="utf-8"))
APP_NAME = METADATA["name"]
PORT = 80
HAPROXY_NAME = "haproxy"
CERTS_NAME = "certificates"
INGRESS_NAME = "ingress"
EXTERNAL_HOSTNAME = "hw.internal"
HAPROXY_EXTERNAL_HOSTNAME = "fqdn.example"


@pytest.mark.juju_setup
def test_deploy(charm: Path, juju: jubilant.Juju):
    """Deploy the charm under test."""
    upstream_source = METADATA["resources"]["hardware-api-image"]["upstream-source"]
    resources = {"hardware-api-image": upstream_source}
    config = {"port": PORT}
    juju.deploy(charm.resolve(), app=APP_NAME, resources=resources, config=config)
    juju.wait(jubilant.all_active)


@pytest.mark.juju_setup
def test_deploy_haproxy(machine_juju: jubilant.Juju):
    """Deploy HAProxy."""
    machine_juju.deploy(
        "haproxy",
        app=HAPROXY_NAME,
        channel="2.8/edge",
        config={"external-hostname": HAPROXY_EXTERNAL_HOSTNAME},
        trust=True,
    )
    machine_juju.deploy("self-signed-certificates", app=CERTS_NAME)
    machine_juju.integrate(f"{HAPROXY_NAME}:certificates", CERTS_NAME)
    machine_juju.wait(jubilant.all_active)
    machine_juju.offer(HAPROXY_NAME, endpoint="haproxy-route")


@pytest.mark.juju_setup
def test_deploy_ingress(juju: jubilant.Juju, machine_juju: jubilant.Juju, machine_controller: str):
    """Deploy the ingress charm."""
    machine_model = model_short_name(machine_juju.model or "")
    juju.consume(f"{machine_model}.{HAPROXY_NAME}", controller=machine_controller)
    juju.deploy(
        "ingress-configurator",
        channel="latest/edge",
        app=INGRESS_NAME,
        config={"hostname": EXTERNAL_HOSTNAME},
        trust=True,
    )
    juju.integrate(INGRESS_NAME, HAPROXY_NAME)
    juju.integrate(f"{APP_NAME}:ingress", INGRESS_NAME)
    juju.wait(jubilant.all_active)


@retry(retry_num=5, retry_sleep_sec=5)
def test_ingress_is_up(machine_juju: jubilant.Juju):
    """Test that the application is reachable via the ingress."""
    ingress_ip = machine_juju.status().apps[HAPROXY_NAME].units[f"{HAPROXY_NAME}/0"].public_address
    session = requests.Session()
    dns_resolver = DNSResolverHTTPAdapter(EXTERNAL_HOSTNAME, ingress_ip)
    session.mount("http://", dns_resolver)
    session.mount("https://", dns_resolver)
    base_url = f"http://{EXTERNAL_HOSTNAME}"
    assert app_is_up(base_url, session=session)


@pytest.mark.juju_teardown
def test_destroy(juju: jubilant.Juju, machine_juju: jubilant.Juju, machine_controller: str):
    """Tear down the charm under test."""
    juju.remove_application(INGRESS_NAME)
    juju.remove_application(APP_NAME)
    juju.cli("remove-saas", "--force", HAPROXY_NAME)
    machine_model = model_short_name(machine_juju.model or "")
    machine_juju.cli(
        "remove-offer",
        f"--controller={machine_controller}",
        "--force",
        "--yes",
        f"{machine_model}.{HAPROXY_NAME}",
        include_model=False,
    )
    machine_juju.remove_application(HAPROXY_NAME)
    machine_juju.remove_application(CERTS_NAME)
