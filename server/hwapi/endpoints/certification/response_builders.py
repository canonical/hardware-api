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

from sqlalchemy.orm import Session

from hwapi.endpoints.certification.response_validators import (
    CertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
    CertifiedImageExistsResponse,
)
from hwapi.data_models import repository, data_validators, models


def build_related_certified_response(
    db: Session,
    machine: models.Machine,
    board: models.Device,
    bios: models.Bios | None,
    releases: list[models.Release],
    kernels: list[models.Kernel],
) -> RelatedCertifiedSystemExistsResponse:
    architecture = repository.get_machine_architecture(db, machine.id)
    bios_validator = (
        data_validators.BiosValidator(
            vendor=bios.vendor.name,
            version=bios.version,
            revision=bios.revision,
            firmware_revision=bios.firmware_revision,
            release_date=(
                bios.release_date.strftime("%m/%d/%Y") if bios.release_date else None
            ),
        )
        if bios
        else None
    )
    return RelatedCertifiedSystemExistsResponse(
        architecture=architecture,
        board=data_validators.BoardValidator(
            manufacturer=board.vendor.name,
            product_name=board.name,
            version=board.version,
        ),
        bios=bios_validator,
        available_releases=[
            data_validators.OSValidator(
                distributor="Ubuntu",
                version=release.release,
                codename=release.codename,
                kernel=data_validators.KernelPackageValidator(
                    name=kernel.name,
                    version=kernel.version,
                    signature=kernel.signature,
                ),
            )
            for release, kernel in zip(releases, kernels)
        ],
    )


def build_certified_response(
    db: Session,
    machine: models.Machine,
    board: models.Device,
    bios: models.Bios | None,
    releases: list[models.Release],
    kernels: list[models.Kernel],
) -> CertifiedResponse:
    architecture = repository.get_machine_architecture(db, machine.id)
    releases, kernels = repository.get_releases_and_kernels_for_machine(db, machine.id)
    bios_validator = (
        data_validators.BiosValidator(
            vendor=bios.vendor.name,
            version=bios.version,
            revision=bios.revision,
            firmware_revision=bios.firmware_revision,
            release_date=(
                bios.release_date.strftime("%m/%d/%Y") if bios.release_date else None
            ),
        )
        if bios
        else None
    )
    return CertifiedResponse(
        architecture=architecture,
        board=data_validators.BoardValidator(
            manufacturer=board.vendor.name,
            product_name=board.name,
            version=board.version,
        ),
        bios=bios_validator,
        available_releases=[
            data_validators.OSValidator(
                distributor="Ubuntu",
                version=release.release,
                codename=release.codename,
                kernel=data_validators.KernelPackageValidator(
                    name=kernel.name,
                    version=kernel.version,
                    signature=kernel.signature,
                ),
            )
            for release, kernel in zip(releases, kernels)
        ],
    )


def build_certified_image_exists_response(
    db: Session,
    machine: models.Machine,
    board: models.Device,
    bios: models.Bios | None,
    releases: list[models.Release],
    kernels: list[models.Kernel],
) -> CertifiedImageExistsResponse:
    architecture = repository.get_machine_architecture(db, machine.id)
    bios_validator = (
        data_validators.BiosValidator(
            vendor=bios.vendor.name,
            version=bios.version,
            revision=bios.revision,
            firmware_revision=bios.firmware_revision,
            release_date=(
                bios.release_date.strftime("%m/%d/%Y") if bios.release_date else None
            ),
        )
        if bios
        else None
    )
    return CertifiedImageExistsResponse(
        architecture=architecture,
        board=data_validators.BoardValidator(
            manufacturer=board.vendor.name,
            product_name=board.name,
            version=board.version,
        ),
        bios=bios_validator,
        available_releases=[
            data_validators.OSValidator(
                distributor="Ubuntu",
                version=release.release,
                codename=release.codename,
                kernel=data_validators.KernelPackageValidator(
                    name=kernel.name,
                    version=kernel.version,
                    signature=kernel.signature,
                ),
            )
            for release, kernel in zip(releases, kernels)
        ],
    )
