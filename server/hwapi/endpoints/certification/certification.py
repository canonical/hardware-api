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


from datetime import datetime, timedelta

from fastapi import APIRouter, Depends

from sqlalchemy.orm import Session
from hwapi.endpoints.certification.rbody_validators import (
    CertificationStatusRequest,
    CertifiedResponse,
    NotCertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
)

from hwapi.data_models import models
from hwapi.data_models import data_validators
from hwapi.data_models.enums import CertificationStatus
from hwapi.data_models.setup import get_db


router = APIRouter()


def is_certified(system_info: CertificationStatusRequest, db: Session):
    """Logic for checking whether system is Certified or not"""
    vendor = (
        db.query(models.Vendor).filter(models.Vendor.name == system_info.vendor).first()
    )
    if not vendor:
        return False, None

    platform = (
        db.query(models.Platform)
        .filter(
            models.Platform.name == system_info.model,
            models.Platform.vendor_id == vendor.id,
        )
        .first()
    )
    if not platform:
        return False, None

    configurations = (
        db.query(models.Configuration)
        .filter(
            models.Configuration.platform_id == platform.id,
        )
        .all()
    )
    if not configurations:
        return False, None

    latest_certificate = None
    latest_release_date = None
    for configuration in configurations:
        machines = (
            db.query(models.Machine)
            .filter(models.Machine.configuration_id == configuration.id)
            .all()
        )
        for machine in machines:
            certificates = (
                db.query(models.Certificate)
                .join(models.Release)
                .filter(models.Certificate.hardware_id == machine.id)
                .order_by(models.Release.release_date.desc())
                .all()
            )
            for certificate in certificates:
                if (
                    not latest_certificate
                    or certificate.release.release_date > latest_release_date
                ):
                    latest_certificate = certificate
                    latest_release_date = certificate.release.release_date

    if latest_certificate:
        return True, CertifiedResponse(
            status=CertificationStatus.CERTIFIED,
            os=data_validators.OSValidator(
                distributor="Canonical Ltd.",
                description="",
                version=latest_certificate.release.release,
                codename=latest_certificate.release.codename,
                kernel=data_validators.KernelPackageValidator(
                    name="", version="", signature=""
                ),
                loaded_modules=[],
            ),
            bios=data_validators.BiosValidator(
                firmware_revision="",
                release_date=datetime.now().date() - timedelta(days=365),
                revision="",
                vendor="",
                version="",
            ),
        )


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
    certified, data = is_certified(system_info, db)
    if certified:
        return data
    return NotCertifiedResponse(status=CertificationStatus.NOT_SEEN)
