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


from fastapi import APIRouter

from hwapi.endpoints.certification.rbody_validators import (
    CertificationStatusRequest,
    CertifiedResponse,
    NotCertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
)


router = APIRouter()


@router.post(
    "/status",
    response_model=(
        CertifiedResponse | NotCertifiedResponse | RelatedCertifiedSystemExistsResponse
    ),
)
def check_certification(system_info: CertificationStatusRequest):
    raise NotImplementedError("The endpoint is not implemented yet")
