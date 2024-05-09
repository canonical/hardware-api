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

from typing import Callable, Any
from sqlalchemy.orm import Session, Query
from pydantic import BaseModel

from hwapi.endpoints.certification.rbody_validators import (
    CertificationStatusRequest,
    CertifiedResponse,
    RelatedCertifiedSystemExistsResponse,
)
from hwapi.data_models import data_validators, repository
from hwapi.data_models.models import Device


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
    Check if some components of the system were seen on other certified machines.
    """
    release = (
        repository.get_release_from_os(db, system_info.os) if system_info.os else None
    )
    board = repository.get_board_by_validator_data(db, system_info.board)
    machines = repository.get_machines_with_same_hardware_params(
        db, system_info.architecture, board, release
    )
    machine_ids = [machine.id for machine in machines]
    devices_query = repository.get_devices_by_machine_ids(db, machine_ids)

    cpus = _get_matching_devices(
        db,
        devices_query,
        system_info.processor or [],
        repository.find_matching_cpu,
        data_validators.ProcessorValidator,
        lambda device, info: data_validators.ProcessorValidator(
            manufacturer=device.vendor.name,
            version=device.name,
            frequency=info.frequency,
            family=info.family,
        ),
    )

    gpus = _get_matching_devices(
        db,
        devices_query,
        system_info.gpu or [],
        repository.find_matching_gpu,
        data_validators.GPUValidator,
        lambda device, info: data_validators.GPUValidator(
            manufacturer=device.vendor.name,
            version=device.name,
            identifier=device.identifier,
        ),
    )

    network_devices = _get_matching_devices(
        db,
        devices_query,
        system_info.network or [],
        repository.find_matching_network_device,
        data_validators.NetworkAdapterValidator,
        lambda device, info: data_validators.NetworkAdapterValidator(
            bus=device.bus,
            identifier=device.identifier,
            model=device.name,
            vendor=device.vendor.name,
            capacity=info.capacity,
        ),
    )

    wireless_devices = _get_matching_devices(
        db,
        devices_query,
        system_info.wireless or [],
        repository.find_matching_wireless_device,
        data_validators.WirelessAdapterValidator,
        lambda device, _: data_validators.WirelessAdapterValidator(
            model=device.name, vendor=device.vendor.name, identifier=device.identifier
        ),
    )

    audio_devices = _get_matching_devices(
        db,
        devices_query,
        system_info.audio or [],
        repository.find_matching_audio_device,
        data_validators.AudioValidator,
        lambda device, _: data_validators.AudioValidator(
            model=device.name, vendor=device.vendor.name, identifier=device.identifier
        ),
    )

    capture_devices = _get_matching_devices(
        db,
        devices_query,
        system_info.video or [],
        repository.find_matching_capture_device,
        data_validators.VideoCaptureValidator,
        lambda device, _: data_validators.VideoCaptureValidator(
            model=device.name, vendor=device.vendor.name, identifier=device.identifier
        ),
    )

    pci_devices = _get_matching_devices(
        db,
        devices_query,
        system_info.pci_peripherals or [],
        repository.find_matching_pci_device,
        data_validators.PCIPeripheralValidator,
        lambda device, _: data_validators.PCIPeripheralValidator(
            name=device.name, vendor=device.vendor.name, pci_id=device.identifier
        ),
    )

    usb_devices = _get_matching_devices(
        db,
        devices_query,
        system_info.usb_peripherals or [],
        repository.find_matching_usb_device,
        data_validators.USBPeripheralValidator,
        lambda device, _: data_validators.USBPeripheralValidator(
            name=device.name, vendor=device.vendor.name, usb_id=device.identifier
        ),
    )

    # If there are no matching devices of any type, return False
    if not any(
        [
            gpus,
            cpus,
            network_devices,
            wireless_devices,
            audio_devices,
            capture_devices,
            pci_devices,
            usb_devices,
        ]
    ):
        return False, None

    return True, RelatedCertifiedSystemExistsResponse(
        architecture=system_info.architecture,
        board=(
            data_validators.BoardValidator(
                manufacturer=board.vendor.name,
                version=board.version,
                product_name=board.name,
            )
            if board
            else None
        ),
        processor=cpus,
        gpu=gpus,
        audio=audio_devices,
        video=capture_devices,
        network=network_devices,
        wireless=wireless_devices,
        pci_peripherals=pci_devices,
        usb_peripherals=usb_devices,
    )


def _get_matching_devices(
    db: Session,
    devices_query: Query,
    device_validator_list: list,
    find_matching_function: Callable[[Session, Query, Any], Device | None],
    device_validator_class: type[BaseModel],
    validator_mapping: Callable[[Device, Any], Any],
):
    """
    General-purpose function to get matching devices.

    :param db: SQLAlchemy database session.
    :param devices_query: Devices query object.
    :param device_validator_list: List of device validator objects.
    :param find_matching_function: The function that retrieves matching devices.
    :param device_validator_class: The validator class to return as response.
    :param validator_mapping: Mapping function to transform devices to validator objects.
    :return: A list of matching device validator objects.
    """
    devices = []
    for device_info in device_validator_list:
        device = find_matching_function(db, devices_query, device_info)
        if device is None:
            continue
        devices.append(validator_mapping(device, device_info))
    return devices
