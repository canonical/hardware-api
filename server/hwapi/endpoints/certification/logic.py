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
)
from hwapi.data_models import data_validators, repository
from hwapi.data_models.enums import CertificationStatus


def is_certified(system_info: CertificationStatusRequest, db: Session):
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

    if latest_certificate:
        return True, CertifiedResponse(
            status=CertificationStatus.CERTIFIED,
            os=data_validators.OSValidator(
                distributor="Canonical Ltd.",
                description="",
                version=latest_certificate.release.release,
                codename=latest_certificate.release.codename,
                kernel=data_validators.KernelPackageValidator(
                    name=kernel.name, version=kernel.version, signature=kernel.signature
                ),
                loaded_modules=[],
            ),
            bios=data_validators.BiosValidator(
                firmware_revision=bios.firmware_version,
                release_date=bios.release_date,
                revision=bios.revision,
                vendor=bios.vendor.name,
                version=bios.version,
            ),
        )
