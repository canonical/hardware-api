# Copyright (C) 2026 Canonical Ltd.
"""Hardware API Charm configuration."""

from typing import Literal

import pydantic


class HardwareApiConfig(pydantic.BaseModel):
    """Hardware API Charm configuration."""

    log_level: Literal["info", "debug", "warning", "error", "critical"] = "info"
    port: int = 30000
