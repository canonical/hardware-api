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
from sqlalchemy import select, and_
from sqlalchemy.orm import Session, selectinload

from hwapi.data_models import models
from hwapi.data_models.enums import DeviceCategory


def _clean_vendor_name(name: str):
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
    stmt = select(model).filter_by(**kwargs)
    instance = db.execute(stmt).scalars().first()
    if instance:
        return instance, False

    if defaults is not None:
        kwargs.update(defaults)

    instance = model(**kwargs)
    db.add(instance)
    db.commit()
    return instance, True


def get_release_object(db: Session, os_version, os_codename) -> models.Release | None:
    """Return release object matching given codename and version"""
    stmt = select(models.Release).filter_by(release=os_version, codename=os_codename)
    return db.execute(stmt).scalars().first()


def get_vendor_by_name(db: Session, name: str) -> models.Vendor | None:
    """Find vendor by name (cleaned-up)"""
    clean_name = f"%{_clean_vendor_name(name).lower()}%"
    stmt = select(models.Vendor).where(models.Vendor.name.ilike(clean_name))
    return db.execute(stmt).scalars().first()


def get_board(
    db: Session, vendor_name: str, product_name: str, version: str
) -> models.Device | None:
    """Return device object (category==BOARD) matching given board data"""
    stmt = (
        select(models.Device)
        .join(models.Vendor)
        .where(
            and_(
                models.Vendor.name.ilike(_clean_vendor_name(vendor_name)),
                models.Device.name.ilike(product_name),
                models.Device.version.ilike(version),
                models.Device.category.in_(
                    [DeviceCategory.BOARD.value, DeviceCategory.OTHER.value]
                ),
            )
        )
    )
    return db.execute(stmt).scalars().first()


def get_bios(
    db: Session, vendor_name: str, version: str, firmware_revision: str | None
) -> models.Bios | None:
    """Return bios object matching given bios data"""
    stmt = (
        select(models.Bios)
        .join(models.Vendor)
        .where(
            and_(
                models.Vendor.name.ilike(_clean_vendor_name(vendor_name)),
                models.Bios.version.ilike(version),
                models.Bios.firmware_revision.ilike(firmware_revision),
            )
        )
    )
    return db.execute(stmt).scalars().first()


def get_machine_with_same_hardware_params(
    db: Session, arch: str, board: models.Device, bios: models.Bios | None
) -> models.Machine | None:
    """
    Get a machines that have the given architecture, motherboard, and optionally bios
    """
    stmt = (
        select(models.Machine)
        .select_from(models.Machine)
        .join(models.Certificate)
        .join(models.Report, models.Certificate.reports)
        .join(models.Device, models.Report.devices)
        .filter(and_(models.Device.id == board.id, models.Report.architecture == arch))
    )

    if bios:
        stmt = stmt.join(models.Bios, models.Report.bios_id == models.Bios.id).filter(
            models.Bios.id == bios.id
        )

    machine = db.execute(stmt.distinct()).scalars().first()
    return machine


def get_machine_by_canonical_id(
    db: Session, canonical_id: str
) -> models.Machine | None:
    stmt = select(models.Machine).where(models.Machine.canonical_id == canonical_id)
    return db.execute(stmt).scalars().first()


def get_machine_architecture(db: Session, machine_id: int) -> str:
    """
    Retrieve the architecture from the latest certificate and the latest report for the specified machine.

    :param db: SQLAlchemy Session instance.
    :param machine_id: integer ID of the machine.
    :return: architecture string if found, None otherwise.
    """
    stmt = (
        select(models.Report.architecture)
        .join(models.Certificate)
        .join(models.Machine)
        .where(models.Machine.id == machine_id)
        .order_by(models.Certificate.created_at.desc())
    )
    result = db.execute(stmt).scalars().first()
    return result if result else ""


def get_certificate_by_name(
    db: Session, machine_id: int, cert_name: str
) -> models.Certificate | None:
    stmt = select(models.Certificate).where(
        and_(
            models.Certificate.name == cert_name,
            models.Certificate.machine_id == machine_id,
        )
    )
    return db.execute(stmt).scalars().first()


def get_cpu_for_machine(db: Session, machine_id: int) -> models.Device | None:
    """
    Retrieve the CPU for the given machine based on the latest report.

    :param db: SQLAlchemy Session instance.
    :param machine_id: integer ID of the machine.
    :return: CPU object if found, None otherwise.
    """
    stmt = (
        select(models.Device)
        .select_from(models.Machine)
        .join(models.Certificate, models.Machine.certificates)
        .join(models.Report, models.Certificate.reports)
        .join(models.Device, models.Report.devices)
        .where(
            and_(
                models.Machine.id == machine_id,
                models.Device.category == DeviceCategory.PROCESSOR,
            )
        )
        .options(selectinload(models.Device.vendor))
        .order_by(models.Certificate.created_at.desc())
    )
    return db.execute(stmt).scalars().first()


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
    stmt = (
        select(models.Release, models.Kernel)
        .join(models.Certificate)
        .join(models.Report)
        .join(models.Kernel)
        .where(models.Certificate.machine_id == machine_id)
        .distinct()
    )
    result = db.execute(stmt).all()
    releases = [release for release, _ in result]
    kernels = [kernel for _, kernel in result]
    return releases, kernels
