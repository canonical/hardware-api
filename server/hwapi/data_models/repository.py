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
from sqlalchemy import and_, true
from sqlalchemy.orm import Session, Query

from hwapi.data_models import models
from hwapi.data_models.enums import DeviceCategory, BusType
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


def get_machines_with_same_hardware_params(
    db: Session,
    arch: str,
    board: models.Device | None,
    release: models.Release | None,
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
                models.Device.id == board.id if board else true,
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
        raise ValueError(
            f"No matching release found for codename {os.codename}, version {os.version}"
        )
    return release


def clean_vendor_name(name):
    """Remove "Inc"/"Inc." substring from vendor name and leading whitespaces"""
    return name.replace("Inc.", "").replace("Inc", "").strip()


def get_board_by_validator_data(
    db: Session, board_validator: device_validators.BoardValidator
) -> models.Device | None:
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
    return board


def get_devices_by_machine_ids(db: Session, machine_ids: list[int]) -> Query:
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


def find_matching_cpu(
    db: Session,
    devices_query: Query,
    cpu_validator: device_validators.ProcessorValidator,
) -> models.Device | None:
    """
    Find a CPU device across the given list of devices that matches have the same
    model name
    """
    return devices_query.filter(
        and_(
            models.Device.name == cpu_validator.version,
            models.Device.category == DeviceCategory.PROCESSOR.value,
        ),
    ).first()


def find_matching_gpu(
    db: Session, devices_query: Query, gpu_validator: device_validators.GPUValidator
) -> models.Device | None:
    """
    Find a GPU device across the given list of devices that have the same model name and
    identifier
    """
    return devices_query.filter(
        and_(
            models.Device.name == gpu_validator.version,
            models.Device.identifier == gpu_validator.identifier.lower(),
            models.Device.category.in_(
                [DeviceCategory.VIDEO.value, DeviceCategory.OTHER.value]
            ),
        )
    ).first()


def find_matching_network_device(
    db: Session,
    devices_query: Query,
    network_validator: device_validators.NetworkAdapterValidator,
) -> models.Device | None:
    """
    Find a network wired device across the given list of devices that has the same
    idenitifier, model name, and bus
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == network_validator.identifier.lower(),
            models.Device.name == network_validator.model,
            models.Device.bus == network_validator.bus,
            models.Device.category.in_(
                [DeviceCategory.NETWORK.value, DeviceCategory.OTHER.value]
            ),
        ),
    ).first()


def find_matching_wireless_device(
    db: Session,
    devices_query: Query,
    wireless_validator: device_validators.WirelessAdapterValidator,
) -> models.Device | None:
    """
    Find a network wireless device across the given list of devices that has the same
    identifier and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == wireless_validator.identifier.lower(),
            models.Device.name == wireless_validator.model,
            models.Device.category.in_(
                [DeviceCategory.WIRELESS.value, DeviceCategory.OTHER.value]
            ),
        ),
    ).first()


def find_matching_audio_device(
    db: Session,
    devices_query: Query,
    audio_validator: device_validators.AudioValidator,
) -> models.Device | None:
    """
    Find a network audio device across the given list of devices that has the same
    identifier and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == audio_validator.identifier.lower(),
            models.Device.name == audio_validator.model,
            models.Device.category.in_(
                [DeviceCategory.AUDIO.value, DeviceCategory.OTHER.value]
            ),
        ),
    ).first()


def find_matching_capture_device(
    db: Session,
    devices_query: Query,
    capture_validator: device_validators.VideoCaptureValidator,
) -> models.Device | None:
    """
    Find a network video capture device across the given list of devices that
    has the same identifier and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == capture_validator.identifier.lower(),
            models.Device.name == capture_validator.model,
            models.Device.category.in_(
                [DeviceCategory.CAPTURE.value, DeviceCategory.OTHER.value]
            ),
        ),
    ).first()


def find_matching_pci_device(
    db: Session,
    devices_query: Query,
    pci_validator: device_validators.PCIPeripheralValidator,
) -> models.Device | None:
    """
    Find an arbitary pci device across the given list of devices that has the same
    PCI ID and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == pci_validator.pci_id.lower(),
            models.Device.bus == BusType.pci.value,
            models.Device.name == pci_validator.name,
        ),
    ).first()


def find_matching_usb_device(
    db: Session,
    devices_query: Query,
    usb_validator: device_validators.USBPeripheralValidator,
) -> models.Device | None:
    """
    Find an arbitary usb device across the given list of devices that has the same
    USB ID and model name
    """
    return devices_query.filter(
        and_(
            models.Device.identifier == usb_validator.usb_id.lower(),
            models.Device.bus == BusType.usb.value,
            models.Device.name == usb_validator.name,
        ),
    ).first()
