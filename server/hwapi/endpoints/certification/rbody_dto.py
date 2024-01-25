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
"""DTO models for request/response bodies"""


from typing import Literal
from pydantic import BaseModel

from hwapi.data_models.enums import CertificationStatus
from hwapi.data_models.models_dto import (
    BiosDTO,
    BoardDTO,
    ChassisDTO,
    CoreDevicesDTO,
    OSDTO,
    PCIPeripheralDTO,
    USBPeripheralDTO,
)


class CertificationStatusRequest(BaseModel):
    os: OSDTO
    bios: BiosDTO
    board: BoardDTO
    chassis: ChassisDTO
    core_devices: CoreDevicesDTO
    pci_peripherals: list[PCIPeripheralDTO]
    usb_peripherals: list[USBPeripheralDTO]


class CertifiedResponse(BaseModel):
    """
    If a system is certified, we return the information about OS and bios
    used on the system under test we had in the lab to certify the machine
    """

    status: Literal[CertificationStatus.CERTIFIED]
    os: OSDTO
    bios: BiosDTO


class NotCertifiedResponse(BaseModel):
    status: Literal[CertificationStatus.NOT_SEEN]


class RelatedCertifiedSystemExistsResponse(BaseModel):
    """
    If a system is partially certified, we return the information about components
    were tested on other systems that the machine has
    """

    status: Literal[CertificationStatus.PARTIALLY_CERTIFIED]
    board: BoardDTO
    chassis: ChassisDTO | None = None
    core_devices: CoreDevicesDTO
    pci_peripherals: list[PCIPeripheralDTO] | None = None
    usb_peripherals: list[USBPeripheralDTO] | None = None
