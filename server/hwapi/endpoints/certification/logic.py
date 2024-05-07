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
"""The algorithms for determining certification status"""

from sqlalchemy.orm import Session

from hwapi.endpoints.certification.rbody_validators import (
    CertificationStatusRequest,
    CertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
)
from hwapi.data_models import data_validators, repository
from hwapi.data_models.enums import CertificationStatus


def is_certified(
    system_info: CertificationStatusRequest, db: Session
) -> tuple[bool, CertifiedResponse | None]:
    """Logic for checking whether system is Certified"""
    configurations = repository.get_configs_by_vendor_and_model(
        db, system_info.vendor, system_info.model
    )
    if configurations is None:
        return False, None

    latest_certificate = repository.get_latest_certificate_for_configs(
        db, configurations
    )
    if latest_certificate is None:
        return False, None

    report = latest_certificate.reports[0]
    kernel = report.kernel
    bios = report.bios

    return True, CertifiedResponse(
        status=CertificationStatus.CERTIFIED,
        os=data_validators.OSValidator(
            distributor="Canonical Ltd.",
            version=latest_certificate.release.release,
            codename=latest_certificate.release.codename,
            kernel=data_validators.KernelPackageValidator(
                name=kernel.name, version=kernel.version, signature=kernel.signature
            ),
            loaded_modules=[],
        ),
        bios=(
            data_validators.BiosValidator(
                release_date=bios.release_date,
                revision=bios.revision,
                firmware_revision=bios.firmware_revision,
                vendor=bios.vendor.name,
                version=bios.version,
            )
            if bios
            else None
        ),
    )


def is_partially_certified(
    system_info: CertificationStatusRequest, db: Session
) -> tuple[bool, RelatedCertifiedSystemExistsResponse | None]:
    """
    Check if some components of the system were seen on other certified machines
    """
    release = (
        repository.get_release_from_os(db, system_info.os) if system_info.os else None
    )
    board = repository.get_board_by_validator_data(db, system_info.board)
    machines = repository.get_machines_with_same_hardware_params(
        db, system_info.architecture, board, release
    )
    machine_ids = [machine.id for machine in machines]

    gpus = []
    if system_info.gpu:
        for gpu_info in system_info.gpu:
            gpu = repository.find_matching_gpu(db, machine_ids, gpu_info)
            if gpu is None:
                continue
            gpus.append(
                data_validators.GPUValidator(
                    manufacturer=gpu.vendor.name,
                    version=gpu.version,
                    identifier=gpu.identifier,
                )
            )
    if not any([gpus]):
        return False, None
    return True, RelatedCertifiedSystemExistsResponse(
        status=CertificationStatus.PARTIALLY_CERTIFIED,
        board=data_validators.BoardValidator(
            manufacturer=board.vendor.name,
            version=board.version,
            product_name=board.name,
        ),
        gpu=gpus,
    )
