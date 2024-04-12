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
"""The endpoints for importing data from C3"""

from fastapi import APIRouter, Depends

from hwapi.data_models.setup import get_db
from hwapi.external.c3.api import C3Api


router = APIRouter()


@router.post("/import-certs", include_in_schema=False)
def import_certs(db_session=Depends(get_db)):
    """Import certificates and related data from C3"""
    c3_api = C3Api(db=db_session)
    c3_api.fetch_certified_configurations()
    return {"status": "OK"}
