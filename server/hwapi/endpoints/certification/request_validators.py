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
"""Vaidator models for request body"""

from pydantic import BaseModel

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
    architecture: str
    board: BoardValidator
    os: OSValidator
    bios: BiosValidator
    chassis: ChassisValidator | None = None
    processor: list[ProcessorValidator] = []
    gpu: list[GPUValidator] | None = None
    audio: list[AudioValidator] | None = None
    video: list[VideoCaptureValidator] | None = None
    network: list[NetworkAdapterValidator] | None = None
    wireless: list[WirelessAdapterValidator] | None = None
    pci_peripherals: list[PCIPeripheralValidator] = []
    usb_peripherals: list[USBPeripheralValidator] = []
