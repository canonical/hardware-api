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


from sqlalchemy import func
from sqlalchemy.orm import Session

from hwapi.data_models import models


def get_configs_by_vendor_and_model(
    db: Session, vendor_name: str, model: str
) -> models.Configuration | None:
    """
    Get configurations for a platform which name is a substring for a "model"
    and that belongs to a vendor with the vendor_name
    """
    vendor = db.query(models.Vendor).filter(models.Vendor.name == vendor_name).first()
    if not vendor:
        return None

    platform = (
        db.query(models.Platform)
        .filter(
            # Check that system model contains platform name
            func.instr(model, models.Platform.name) > 0,
            models.Platform.vendor_id == vendor.id,
        )
        .first()
    )
    if not platform:
        return None

    return platform.configurations


def get_latest_certificate_for_configs(
    db: Session, configurations: list[models.Configuration]
) -> models.Certificate | None:
    """For a given list of configurations, find the latest certificate"""
    latest_certificate = None
    latest_release_date = None
    for configuration in configurations:
        for machine in configuration.machines:
            certificates = (
                db.query(models.Certificate)
                .join(models.Release)
                .filter(models.Certificate.machine_id == machine.id)
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

    if latest_certificate is None or not latest_certificate.reports:
        return None
    return latest_certificate
