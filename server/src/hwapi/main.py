# Copyright 2024 Canonical Ltd.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License version
# 3, as published by the Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# Written by:
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>

import logging
import os

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from hwapi.router import router

logging.basicConfig(format="%(asctime)s - %(name)s - %(levelname)s - %(message)s")

app = FastAPI(
    title="Hardware API (hwapi)",
    version="1.0.0",
    description="API server for working with hardware information from C3",
    redirect_slashes=False,
)

_allowed_origins = [
    o.strip()
    for o in os.environ.get("HWAPI_ALLOWED_ORIGINS", "").split(",")
    if o.strip()
]
if _allowed_origins:
    app.add_middleware(
        CORSMiddleware,  # type: ignore[invalid-argument-type]
        allow_origins=_allowed_origins,
        allow_methods=["GET", "POST"],
        allow_headers=["Content-Type"],
    )


app.include_router(router)
