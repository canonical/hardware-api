# Copyright 2024 Canonical Ltd.
# All rights reserved.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# Written by:
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>

import os
import logging

from fastapi import Request, HTTPException


logger = logging.getLogger(__name__)


def only_internal_hosts(request: Request):
    """Decorator for internal endpoints to accept requests only from localhost"""
    # Issue: https://github.com/tiangolo/fastapi/discussions/11019, updating dependencies
    # doesn't resolve the problem
    client_host = request.client.host  # type: ignore [union-attr]
    internal_hosts = os.getenv("INTERNAL_HOSTS", "127.0.0.1,::1")
    if internal_hosts == "*":
        return True
    if client_host not in internal_hosts.split(","):
        logger.warning(
            "Got a request to a private endpoint from forbidden host %s", client_host
        )
        raise HTTPException(status_code=403, detail="Access forbidden")
    return True
