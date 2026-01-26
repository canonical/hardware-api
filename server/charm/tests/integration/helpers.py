# Copyright (C) 2026 Canonical
"""Helper classes and functions for integration testing."""

import functools
import logging
import time
from urllib.parse import urlparse

import requests
from requests.adapters import HTTPAdapter

logger = logging.getLogger(__name__)


class DNSResolverHTTPAdapter(HTTPAdapter):
    """Simple DNS resolver for HTTP requests."""

    def __init__(self, hostname: str, ip: str, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.hostname = hostname
        self.ip = ip

    def send(self, request, stream=False, timeout=None, verify=True, cert=None, proxies=None):
        result = urlparse(request.url)
        if result.hostname == self.hostname and result.scheme in ("http", "https") and self.ip:
            logger.debug("DNS substitution: '%s' to '%s'", result.hostname, self.ip)
            request.url = request.url.replace(  # type: ignore
                f"{result.scheme}://{result.hostname}", f"{result.scheme}://{self.ip}"
            )
            request.headers["Host"] = self.hostname
            if result.scheme == "https":
                self.poolmanager.connection_pool_kw["server_hostname"] = self.hostname
                self.poolmanager.connection_pool_kw["assert_hostname"] = self.hostname
        elif result.hostname == self.hostname:
            self.poolmanager.connection_pool_kw.pop("server_hostname", None)
            self.poolmanager.connection_pool_kw.pop("assert_hostname", None)
        return super().send(request, stream, timeout, verify, cert, proxies)


def app_is_up(base_url: str, session: requests.Session | None = None) -> bool:
    """Check that the hwapi application is up."""
    base_url = base_url.rstrip("/")
    endpoint = "/"
    url = f"{base_url}{endpoint}"
    logger.info("Querying: %s", url)
    get = session.get if session else requests.get
    response = get(url, timeout=15, verify=False)
    ok = response.ok and "Hardware Information API (hwapi) server" in response.text
    if not ok:
        logger.error("Response: %s", response)
    return ok


def retry(retry_num: int, retry_sleep_sec: int):
    """Retry function decorator."""

    def decorator(func):
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            for i in range(retry_num):
                try:
                    return func(*args, **kwargs)
                except Exception as exc:
                    if i >= retry_num - 1:
                        raise Exception(f"Exceeded {retry_num} retries") from exc
                    logger.error("func %s failure %d/%d: %s", func.__name__, i + 1, retry_num, exc)
                    time.sleep(retry_sleep_sec)

        return wrapper

    return decorator
