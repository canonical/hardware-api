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
"""The endpoints for working with certification status"""


from fastapi import APIRouter, Depends
from fastapi.responses import JSONResponse
from sqlalchemy.orm import Session

from hwapi.endpoints.certification.rbody_validators import (
    CertificationStatusRequest,
    CertifiedResponse,
    NotCertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
)
from hwapi.data_models.setup import get_db

from .logic import is_certified, is_partially_certified


router = APIRouter()


@router.post(
    "/status",
    response_model=(
        CertifiedResponse | NotCertifiedResponse | RelatedCertifiedSystemExistsResponse
    ),
)
def check_certification(
    system_info: CertificationStatusRequest, db: Session = Depends(get_db)
):
    """
    Endpoint for checking certification status (whether a system is certified, not seen
    or some of its components have been seen on other systems)
    """
    data: CertifiedResponse | RelatedCertifiedSystemExistsResponse | None
    certified, data = is_certified(system_info, db)
    if certified:
        return data
    try:
        partially_certifed, data = is_partially_certified(system_info, db)
    except ValueError as exc:
        return JSONResponse(status_code=400, content={"detail": str(exc)})
    if partially_certifed:
        return data
    return NotCertifiedResponse()
