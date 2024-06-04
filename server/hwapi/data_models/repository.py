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


from typing import Any
from sqlalchemy import and_
from sqlalchemy.orm import Session, Query

from hwapi.data_models import models
from hwapi.data_models.enums import DeviceCategory, BusType
from hwapi.data_models.data_validators.software import OSValidator


def clean_vendor_name(name: str):
    """Remove "Inc"/"Inc." substring from vendor name and leading whitespaces"""
    return name.replace("Inc.", "").replace("Inc", "").strip()


def get_configs_by_vendor_and_model(
    db: Session, vendor_name: str, model: str
) -> list[models.Configuration] | None:
    """
    Get configurations for a platform which name (without data in parenthesis) is a
    substring of a model and that belongs to a vendor with the vendor_name
    """
    vendor = db.query(models.Vendor).filter(models.Vendor.name == vendor_name).first()
    if not vendor:
        return None

    filtered_platform = None

    for platform in vendor.platforms:
        # Ignore data in parenthesis
        platform_name = platform.name
        parenth_idx = platform_name.find("(")
        if parenth_idx != -1:
            platform_name = platform_name[:parenth_idx].strip()
        if platform_name in model:
            filtered_platform = platform

    return filtered_platform.configurations if filtered_platform else None


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


def get_or_create(
    db: Session,
    model: type[models.Base],
    defaults: dict[str, Any] | None = None,
    **kwargs,
) -> tuple:
    """
    Retrieves an object from the database based on the provided kwargs. If it doesn't
    exist, it creates the object using both the kwargs and any additional default
    attributes provided.

    :param db: Database session
    :param model: SQLAlchemy Model class
    :param defaults: Dictionary of default values to initialise the model
    :param kwargs: Keyword arguments corresponding to the model's attributes used for
                   lookup and creation
    :return: A tuple of (instance, created), where `created` is a boolean indicating
             whether the instance was created in this call
    """
    instance = db.query(model).filter_by(**kwargs).first()
    if instance:
        return instance, False

    if defaults is not None:
        kwargs.update(defaults)

    instance = model(**kwargs)
    db.add(instance)
    db.commit()
    return instance, True


def get_release_from_os(db: Session, os: OSValidator) -> models.Release | None:
    """Return release object matching given OS data"""
    return (
        db.query(models.Release)
        .filter_by(release=os.version, codename=os.codename)
        .first()
    )


def get_vendor_by_name(db: Session, name: str) -> models.Vendor | None:
    """Find vendor by name (cleaned-up)"""
    return (
        db.query(models.Vendor)
        .filter(models.Vendor.name.ilike(f"%{clean_vendor_name(name).lower()}%"))
        .first()
    )


def get_board(
    db: Session, vendor_name: str, product_name: str, version: str
) -> models.Device | None:
    """Return device object (category==BOARD) matching given board data"""
    return (
        db.query(models.Device)
        .join(models.Vendor)
        .filter(
            and_(
                models.Vendor.name.ilike(vendor_name),
                models.Device.name.ilike(product_name),
                models.Device.version.ilike(version),
                models.Device.category.in_(
                    [DeviceCategory.BOARD.value, DeviceCategory.OTHER.value]
                ),
            )
        )
        .first()
    )


def get_bios(db: Session, vendor_name: str, version: str) -> models.Bios | None:
    """Return bios object matching given bios data"""
    return (
        db.query(models.Bios)
        .join(models.Vendor)
        .filter(
            and_(
                models.Vendor.name.ilike(vendor_name),
                models.Bios.version.ilike(version),
            )
        )
        .first()
    )


def get_machines_devices_query(db: Session, machine_ids: list[int]) -> Query:
    """
    Return query that contains Devices joined with Reports and Certificates for a
    given list of machine IDs
    """
    return (
        db.query(models.Device)
        .join(
            models.device_report_association,
            models.Device.id == models.device_report_association.c.device_id,
        )
        .join(
            models.Report,
            models.device_report_association.c.report_id == models.Report.id,
        )
        .join(models.Certificate, models.Report.certificate_id == models.Certificate.id)
        .filter(models.Certificate.machine_id.in_(machine_ids))
    )


def get_machines_with_same_hardware_params(
    db: Session, arch: str, board: models.Device, bios: models.Bios
) -> list[models.Machine]:
    """
    Retrieve all the machines that have the given architecture, motherboard (optionally),
    and are certified for the given release (if specified)
    """
    return (
        db.query(models.Machine)
        .join(models.Certificate)
        .join(models.Report, models.Certificate.reports)
        .join(
            models.device_report_association,
            models.Report.id == models.device_report_association.c.report_id,
        )
        .join(
            models.Device,
            models.device_report_association.c.device_id == models.Device.id,
        )
        .filter(
            and_(
                models.Device.id == board.id,
                models.Report.architecture == arch,
                models.Report.bios_id == bios.id,
            )
        )
        .distinct()
        .all()
    )


def get_processors_by_family(
    db: Session,
    family: str
) -> list[models.Device]:
    """
    Find a CPU device across the given list of devices that contains
    model name.
    """
    return db.query(models.Device).filter(
        and_(
            models.Device.family.ilike(family),
            models.Device.category == DeviceCategory.PROCESSOR,
        ),
    ).distinct().all()


def get_matching_pci_device(
    db: Session,
    devices_query: Query,
    identifier: str,
) -> models.Device | None:
    """
    Find an arbitary pci device across the given list of devices that has the same
    PCI ID and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == identifier.lower(),
            models.Device.bus == BusType.pci,
        ),
    ).first()


def get_matching_usb_device(
    db: Session,
    devices_query: Query,
    identifier: str,
) -> models.Device | None:
    """
    Find an arbitary usb device across the given list of devices that has the same
    USB ID and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == identifier.lower(),
            models.Device.bus == BusType.usb,
        ),
    ).first()
