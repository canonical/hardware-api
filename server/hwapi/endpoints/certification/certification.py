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

from hwapi.endpoints.certification.request_validators import CertificationStatusRequest
from hwapi.endpoints.certification.response_validators import (
    CertifiedResponse,
    NotCertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
    CertifiedImageExists,
)
from hwapi.data_models import repository, data_validators
from hwapi.data_models.setup import get_db


router = APIRouter()


@router.post(
    "/status",
    response_model=(
        CertifiedResponse
        | NotCertifiedResponse
        | RelatedCertifiedSystemExistsResponse
        | CertifiedImageExists
    ),
)
def check_certification(
    system_info: CertificationStatusRequest, db: Session = Depends(get_db)
) -> (
    CertifiedResponse
    | NotCertifiedResponse
    | RelatedCertifiedSystemExistsResponse
    | CertifiedImageExists
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
    board = repository.get_board(
        db,
        system_info.board.manufacturer,
        system_info.board.product_name,
        system_info.board.version,
    )
    bios = repository.get_bios(db, system_info.bios.vendor, system_info.bios.version)
    if board is None or bios is None:
        return NotCertifiedResponse()
    related_machine = repository.get_machine_with_same_hardware_params(
        db, system_info.architecture, board, bios
    )
    if related_machine is None:
        return NotCertifiedResponse()
    related_machine_cpu = repository.get_cpu_for_machine(db, related_machine.id)
    if related_machine_cpu is None:
        return NotCertifiedResponse()
    # Match against CPU family
    if related_machine_cpu.family.lower() != system_info.processor.family.lower():
        return RelatedCertifiedSystemExistsResponse(
            architecture=repository.get_machine_architecture(db, related_machine.id),
            board=data_validators.BoardValidator(
                manufacturer=board.vendor.name,
                product_name=board.name,
                version=board.version,
            ),
            bios=data_validators.BiosValidator(
                vendor=bios.vendor.name,
                version=bios.version,
                revision=bios.revision,
                firmware_revision=bios.firmware_revision,
                release_date=bios.release_date,
            ),
        )
    # Check OS release
    release_from_request = repository.get_release_from_os(
        db, system_info.os.version, system_info.os.codename
    )
    related_releases, kernels = repository.get_releases_and_kernels_for_machine(
        db, related_machine.id
    )
    if release_from_request in related_releases:
        return CertifiedResponse(
            architecture=repository.get_machine_architecture(db, related_machine.id),
            board=data_validators.BoardValidator(
                manufacturer=board.vendor.name,
                product_name=board.name,
                version=board.version,
            ),
            bios=data_validators.BiosValidator(
                vendor=bios.vendor.name,
                version=bios.version,
                revision=bios.revision,
                firmware_revision=bios.firmware_revision,
                release_date=bios.release_date,
            ),
            available_releases=[
                data_validators.OSValidator(
                    distributor="Ubuntu",
                    version=os.release,
                    codename=os.codename,
                    kernel=data_validators.KernelPackageValidator(
                        name=kernels[i].name,
                        version=kernels[i].version,
                        signature=kernels[i].signature,
                    ),
                )
                for i, os in enumerate(related_releases)
            ],
        )
    return CertifiedImageExists(
        architecture=repository.get_machine_architecture(db, related_machine.id),
        board=data_validators.BoardValidator(
            manufacturer=board.vendor.name,
            product_name=board.name,
            version=board.version,
        ),
        bios=data_validators.BiosValidator(
            vendor=bios.vendor.name,
            version=bios.version,
            revision=bios.revision,
            firmware_revision=bios.firmware_revision,
            release_date=bios.release_date,
        ),
        available_releases=[
            data_validators.OSValidator(
                distributor="Ubuntu",
                version=os.release,
                codename=os.codename,
                kernel=data_validators.KernelPackageValidator(
                    name=kernels[i].name,
                    version=kernels[i].version,
                    signature=kernels[i].signature,
                ),
            )
            for i, os in enumerate(related_releases)
        ],
    )
