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
    USBPeripheralValidator,
    VideoCaptureValidator,
    WirelessAdapterValidator,
)


class CertifiedResponse(BaseModel):
    status: Literal[CertificationStatus.CERTIFIED] = CertificationStatus.CERTIFIED
    architecture: str
    bios: BiosValidator | None
    board: BoardValidator
    chassis: ChassisValidator | None = None
    available_releases: list[OSValidator]


class NotCertifiedResponse(BaseModel):
    status: Literal[CertificationStatus.NOT_SEEN] = CertificationStatus.NOT_SEEN


class RelatedCertifiedSystemExistsResponse(BaseModel):
    status: Literal[CertificationStatus.RELATED_CERTIFIED_SYSTEM_EXISTS] = (
        CertificationStatus.RELATED_CERTIFIED_SYSTEM_EXISTS
    )
    architecture: str
    board: BoardValidator
    bios: BiosValidator | None
    chassis: ChassisValidator | None = None
    gpu: list[GPUValidator] | None = None
    audio: list[AudioValidator] | None = None
    video: list[VideoCaptureValidator] | None = None
    network: list[NetworkAdapterValidator] | None = None
    wireless: list[WirelessAdapterValidator] | None = None
    pci_peripherals: list[PCIPeripheralValidator] = []
    usb_peripherals: list[USBPeripheralValidator] = []
    available_releases: list[OSValidator]


class CertifiedImageExistsResponse(BaseModel):
    status: Literal[CertificationStatus.CERTIFIED_IMAGE_EXISTS] = (
        CertificationStatus.CERTIFIED_IMAGE_EXISTS
    )
    architecture: str
    bios: BiosValidator | None
    board: BoardValidator
    chassis: ChassisValidator | None = None
    available_releases: list[OSValidator]
