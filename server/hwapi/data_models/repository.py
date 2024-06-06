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


def clean_vendor_name(name: str):
    """Remove "Inc"/"Inc." substring from vendor name and leading whitespaces"""
    return name.replace("Inc.", "").replace("Inc", "").strip()


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


def get_release_from_os(db: Session, os_version, os_codename) -> models.Release | None:
    """Return release object matching given OS data"""
    return (
        db.query(models.Release)
        .filter_by(release=os_version, codename=os_codename)
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


def get_machine_with_same_hardware_params(
    db: Session, arch: str, board: models.Device, bios: models.Bios
) -> models.Machine | None:
    """
    Get a machines that have the given architecture, motherboard, bios
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
        .first()
    )


def get_machine_architecture(db: Session, machine_id: int) -> str:
    """
    Retrieve the architecture from the latest certificate and the latest report for the specified machine.

    :param db: SQLAlchemy Session instance.
    :param machine_id: integer ID of the machine.
    :return: architecture string if found, None otherwise.
    """
    latest_report = (
        db.query(models.Report)
        .join(models.Certificate, models.Report.certificate_id == models.Certificate.id)
        .join(models.Machine, models.Certificate.machine_id == models.Machine.id)
        .filter(models.Machine.id == machine_id)
        .order_by(models.Certificate.created_at.desc())
        .first()
    )

    return latest_report.architecture if latest_report else ""


def get_cpu_for_machine(db: Session, machine_id: int) -> models.Device | None:
    """
    Retrieve the CPU codename for the given machine based on the latest report.

    :param db: SQLAlchemy Session instance.
    :param machine_id: integer ID of the machine.
    :return: CPU codename string if found, None otherwise.
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
        .filter(
            models.Certificate.machine_id == machine_id,
            models.Device.category == DeviceCategory.PROCESSOR.value,
        )
        .order_by(models.Certificate.created_at.desc())
        .first()
    )


def get_releases_and_kernels_for_machine(
    db: Session, machine_id: int
) -> tuple[list[models.Release], list[models.Kernel]]:
    """
    Retrieve all distinct releases and their corresponding kernel information for which a
    given machine has been certified.

    :param db: SQLAlchemy Session instance.
    :param machine_id: integer ID of the machine.
    :returns: a list of tuples, each containing a Release model instance and the kernel version string.
    """
    result = (
        db.query(models.Release, models.Kernel)
        .join(models.Certificate, models.Release.id == models.Certificate.release_id)
        .join(models.Report, models.Certificate.id == models.Report.certificate_id)
        .join(models.Kernel, models.Report.kernel_id == models.Kernel.id)
        .filter(models.Certificate.machine_id == machine_id)
        .distinct()
        .all()
    )
    releases = []
    kernels = []
    for release, kernel in result:
        releases.append(release)
        kernels.append(kernel)
    return releases, kernels


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
