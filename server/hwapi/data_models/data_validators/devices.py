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


class AudioValidator(BaseModel):
    """Validator model for audio devices"""

    model: str
    vendor: str


class BiosValidator(BaseModel):
    firmware_revision: str
    release_date: date
    revision: str
    vendor: str
    version: str


class BoardValidator(BaseModel):
    manufacturer: str
    product_name: str
    version: str


class ChassisValidator(BaseModel):
    chassis_type: str
    manufacturer: str
    version: str


class GPUValidator(BaseModel):
    family: str
    manufacturer: str
    version: str


class NetworkAdapterValidator(BaseModel):
    """Validator for ethernet network adapters"""

    model: str
    vendor: str
    capacity: int


class PCIPeripheralValidator(BaseModel):
    pci_id: str
    name: str
    vendor: str


class ProcessorValidator(BaseModel):
    family: str
    frequency: float
    manufacturer: str
    version: str


class USBPeripheralValidator(BaseModel):
    usb_id: str
    name: str
    vendor: str


class VideoCaptureValidator(BaseModel):
    """Validator model for video capture devices"""

    model: str
    vendor: str


class WirelessAdapterValidator(BaseModel):
    """Validator model for wireless network adapters"""

    model: str
    vendor: str
