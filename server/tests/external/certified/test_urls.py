"""Tests for the ubuntu.com/certified URLs."""

from hwapi.external.certified.urls import (
    get_certified_configuration_url,
    get_certified_platform_url,
)


def test_get_certified_platform_url():
    """Tests that the public platform URL is valid."""
    platform_name = "Some Platform Name"
    url = get_certified_platform_url(platform_name)
    assert url == "https://ubuntu.com/certified?q=Some+Platform+Name"


def test_get_certified_configuration_url():
    """Tests that the public configuration URL is valid."""
    canonical_id = "202512-30000"
    url = get_certified_configuration_url(canonical_id)
    assert url == f"https://ubuntu.com/certified/{canonical_id}"
