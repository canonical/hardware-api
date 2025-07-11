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
"""The endpoints for working with certification status"""

import logging
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
        logging.warning("Failed to match vendor: %s", system_info.vendor)
        return NotCertifiedResponse()

    # Match against board and bios
    try:
        board = logic.find_board(db, system_info.board)
        bioses = logic.find_bioses(db, system_info.bios) if system_info.bios else []
        related_machine = logic.find_certified_machine(
            db, system_info.architecture, board, bioses
        )
    except ValueError:
        logging.warning(
            (
                "Hardware cannot be found. Machine vendor: %s, model: %s"
                ", board model: %s, board version: %s, bios version: %s"
            ),
            system_info.vendor,
            system_info.model,
            system_info.board.product_name,
            system_info.board.version,
            system_info.bios.version if system_info.bios else None,
        )
        return NotCertifiedResponse()

    bios = repository.get_machine_bios(db, related_machine.id)
    related_releases, kernels = repository.get_releases_and_kernels_for_machine(
        db, related_machine.id
    )

    # Match against CPU codename
    if not logic.check_cpu_compatibility(db, related_machine, system_info.processor):
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
