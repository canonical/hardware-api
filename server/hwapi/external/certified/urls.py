"""This module contains the ubuntu.com/certified URLs."""

BASE_URL = "https://ubuntu.com/certified"


def get_certified_platform_url(platform_id: int) -> str:
    """Build the certified platform URL."""
    return f"{BASE_URL}/platforms/{platform_id}"


def get_certified_configuration_url(canonical_id: str) -> str:
    """Build the certified configuration URL.

    Currently ubuntu.com/certified doesn't have a true configuration page,
    instead using the Canonical ID of the machine linked to a certificate.
    """
    return f"{BASE_URL}/{canonical_id}"
