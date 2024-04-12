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
"""Validator models for request/response bodies"""


from typing import Literal
from pydantic import BaseModel

from hwapi.data_models.enums import CertificationStatus
from hwapi.data_models.data_validators import (
    AudioValidator,
    BiosValidator,
    BoardValidator,
    ChassisValidator,
    GPUValidator,
    NetworkAdapterValidator,
    OSValidator,
    PCIPeripheralValidator,
    ProcessorValidator,
    USBPeripheralValidator,
    VideoCaptureValidator,
    WirelessAdapterValidator,
)


class CertificationStatusRequest(BaseModel):
    """Request body validator for status check endpoint"""

    vendor: str
    model: str
    os: OSValidator | None = None
    bios: BiosValidator | None = None
    board: BoardValidator | None = None
    chassis: ChassisValidator | None = None
    processor: list[ProcessorValidator] = []
    gpu: list[GPUValidator] | None = None
    audio: list[AudioValidator] | None = None
    video: list[VideoCaptureValidator] | None = None
    network: list[NetworkAdapterValidator] | None = None
    wireless: list[WirelessAdapterValidator] | None = None
    pci_peripherals: list[PCIPeripheralValidator] = []
    usb_peripherals: list[USBPeripheralValidator] = []


class CertifiedResponse(BaseModel):
    """
    If a system is certified, we return the information about OS and bios
    used on the system under test we had in the lab to certify the machine
    """

    status: Literal[CertificationStatus.CERTIFIED]
    os: OSValidator
    bios: BiosValidator | None


class NotCertifiedResponse(BaseModel):
    status: Literal[CertificationStatus.NOT_SEEN]


class RelatedCertifiedSystemExistsResponse(BaseModel):
    """
    If a system is partially certified, we return the information about components
    were tested on other systems that the machine has
    """

    status: Literal[CertificationStatus.PARTIALLY_CERTIFIED]
    board: BoardValidator
    chassis: ChassisValidator | None = None
    processor: list[ProcessorValidator] | None = None
    gpu: list[GPUValidator] | None = None
    audio: list[AudioValidator] | None = None
    video: list[VideoCaptureValidator] | None = None
    network: list[NetworkAdapterValidator] | None = None
    wireless: list[WirelessAdapterValidator] | None = None
    pci_peripherals: list[PCIPeripheralValidator] | None = None
    usb_peripherals: list[USBPeripheralValidator] | None = None
