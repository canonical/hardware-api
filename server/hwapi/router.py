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


import yaml
from fastapi import APIRouter, Response
from fastapi.openapi.utils import get_openapi
from .endpoints.certification import certification


router = APIRouter()
router.include_router(
    certification.router, prefix="/v1/certification", tags=["certification"]
)


@router.get("/")
def root():
    return "Hardware Information API (hwapi) server"


@router.get("/v1/openapi.yaml", include_in_schema=False)
def get_openapi_yaml():
    """OpenAPI schema in YAML format"""
    openapi_schema = get_openapi(
        title="Hardware API (hwapi)",
        version="1.0.0",
        description="API server for working with hardware information from C3",
        routes=router.routes,
    )
    return Response(content=yaml.dump(openapi_schema), media_type="application/yaml")
