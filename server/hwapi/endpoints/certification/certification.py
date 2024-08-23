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
from sqlalchemy.orm import Session

from hwapi.data_models import repository
from hwapi.data_models.setup import get_db
from hwapi.endpoints.certification import logic, response_builders
from hwapi.endpoints.certification.request_validators import CertificationStatusRequest
from hwapi.endpoints.certification.response_validators import (
    CertifiedResponse,
    NotCertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
    CertifiedImageExistsResponse,
)

router = APIRouter()


@router.post(
    "/status",
    response_model=(
        CertifiedResponse
        | NotCertifiedResponse
        | RelatedCertifiedSystemExistsResponse
        | CertifiedImageExistsResponse
    ),
)
def check_certification(
    system_info: CertificationStatusRequest, db: Session = Depends(get_db)
) -> (
    CertifiedResponse
    | NotCertifiedResponse
    | RelatedCertifiedSystemExistsResponse
    | CertifiedImageExistsResponse
):
    """
    Endpoint for checking certification status (whether a system is certified, not seen
    or some of its components have been seen on other systems)
    """
    # Check against vendor
    vendor = repository.get_vendor_by_name(db, system_info.vendor)
    if not vendor:
        return NotCertifiedResponse()
    # Match against board and bios
    try:
        board, bios = logic.find_main_hardware_components(
            db, system_info.board, system_info.bios
        )
        related_machine = logic.find_certified_machine(
            db, system_info.architecture, board, bios
        )
    except ValueError:
        return NotCertifiedResponse()
    related_releases, kernels = repository.get_releases_and_kernels_for_machine(
        db, related_machine.id
    )
    # Match against CPU codename
    if not logic.check_cpu_compatibility(
        db, related_machine, system_info.processor.cpu_id
    ):
        return response_builders.build_related_certified_response(
            db, related_machine, board, bios, related_releases, kernels
        )
    # Check OS release
    release_from_request = repository.get_release_object(
        db, system_info.os.version, system_info.os.codename
    )
    if release_from_request in related_releases:
        # If machine has been certified for the specified in the request release
        # return Certified response
        return response_builders.build_certified_response(
            db, related_machine, board, bios, related_releases, kernels
        )
    return response_builders.build_certified_image_exists_response(
        db, related_machine, board, bios, related_releases, kernels
    )
