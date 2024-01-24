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


from datetime import date
from pydantic import BaseModel


# Devices


class BiosDTO(BaseModel):
    firmware_revision: str
    release_date: date
    revision: str
    vendor: str
    version: str


class BoardDTO(BaseModel):
    manufacturer: str
    product_name: str
    version: str


class ChassisDTO(BaseModel):
    chassis_type: str
    manufacturer: str
    version: str


class ProcessorDTO(BaseModel):
    family: str
    frequency: float
    manufacturer: str
    version: str


class GPUDTO(BaseModel):
    family: str
    manufacturer: str
    version: str


class NetworkAdapterDTO(BaseModel):
    """DTO for ethernet network adapters"""

    model: str
    vendor: str
    capacity: int


class WirelessAdapterDTO(BaseModel):
    """DTO model for wireless network adapters"""

    model: str
    vendor: str


class AudioDTO(BaseModel):
    """DTO model for audio devices"""

    model: str
    vendor: str


class VideoCaptureDTO(BaseModel):
    """DTO model for video capture devices"""

    model: str
    vendor: str


class CoreDevicesDTO(BaseModel):
    """
    DTO for core devices that includes:
      - cpu
      - gpu
      - ethernet and wireless network adapters
      - audio controllers
      - video capture devices

    All of them are marked optional since some of them may not be presented
    in the list of seen/tested/certified devices
    """

    processor: ProcessorDTO | None
    gpu: GPUDTO | None
    network: list[NetworkAdapterDTO] | None
    wireless: list[WirelessAdapterDTO] | None
    audio: list[AudioDTO] | None
    video: list[VideoCaptureDTO] | None


class PCIPeripheralDTO(BaseModel):
    pci_id: str
    name: str
    vendor: str


class USBPeripheralDTO(BaseModel):
    usb_id: str
    name: str
    vendor: str


# Software


class PackageDTO(BaseModel):
    name: str
    version: str


class OSDTO(BaseModel):
    distributor: str
    description: str
    version: str
    codename: str
    kernel: PackageDTO
    loaded_modules: list[str]
