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


from typing import Type, Any
from sqlalchemy import and_, or_, true
from sqlalchemy.orm import Session

from hwapi.data_models import models
from hwapi.data_models.enums import DeviceCategory
from hwapi.data_models.data_validators.software import OSValidator
from hwapi.data_models.data_validators import devices as device_validators


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
    model: Type[models.Base],
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


def get_machines_with_same_hardware_params(
    db: Session,
    arch: str,
    board: models.Device,
    release: models.Release | None,
) -> list[models.Machine]:
    """
    Retrieve all the machines that have the given board and architecture and are
    certified for the given release
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
                (
                    models.Certificate.release_id == release.id
                    if release is not None
                    else true
                ),
            )
        )
        .distinct()
        .all()
    )


def get_release_from_os(db: Session, os: OSValidator) -> models.Release:
    """Return release object matching given OS data"""
    release = (
        db.query(models.Release)
        .filter_by(release=os.version, codename=os.codename)
        .first()
    )
    if release is None:
        raise ValueError(f"No matching release found for {os.version}")
    return release


def clean_vendor_name(name):
    """Remove "Inc"/"Inc." substring from vendor name and leading whitespaces"""
    return name.replace("Inc.", "").replace("Inc", "").strip()


def get_board_by_validator_data(
    db: Session, board_validator: device_validators.BoardValidator
) -> models.Device:
    """Return device object (category==BOARD) matching given Board data"""
    board = (
        db.query(models.Device)
        .join(models.Vendor)
        .filter(
            and_(
                models.Vendor.name.in_(
                    [
                        board_validator.manufacturer,
                        clean_vendor_name(board_validator.manufacturer),
                    ]
                ),
                models.Device.name == board_validator.product_name,
                models.Device.version == board_validator.version,
                models.Device.category.in_(
                    [DeviceCategory.BOARD.value, DeviceCategory.OTHER.value]
                ),
            )
        )
        .first()
    )
    if board is None:
        raise ValueError(
            f"Matching board was not found for board {board_validator.product_name}"
        )
    return board


def find_matching_gpu(
    db: Session, machine_ids: list[int], gpu_validator: device_validators.GPUValidator
) -> models.Device | None:
    """
    Find a GPU device across the given list of machines that matches the parameters
    specified in the GPUValidator
    """
    gpu = (
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
            and_(
                models.Certificate.machine_id.in_(machine_ids),
                models.Device.name.ilike(f"%{gpu_validator.family}%"),
                models.Device.vendor.has(
                    or_(
                        models.Vendor.name == gpu_validator.manufacturer,
                        models.Vendor.name == clean_vendor_name(gpu_validator.manufacturer)
                    )
                ),
                (
                    models.Device.version == gpu_validator.version
                    if gpu_validator.version is not None
                    else true
                ),
                models.Device.identifier == gpu_validator.identifier,
                models.Device.category.in_(
                    [DeviceCategory.VIDEO.value, DeviceCategory.OTHER.value]
                ),
            )
        )
        .first()
    )

    return gpu
