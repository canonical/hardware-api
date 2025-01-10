# Copyright 2024 Canonical Ltd.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License version
# 3, as published by the Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# Written by:
#        Nadzeya Hutsko <nadzeya.hutsko@canonical.com>
"""Vaidator models for request body"""

from pydantic import BaseModel

from hwapi.data_models.data_validators import (
    BiosValidator,
    BoardValidator,
    ChassisValidator,
    OSValidator,
    PCIPeripheralValidator,
    ProcessorValidator,
    USBPeripheralValidator,
)


class CertificationStatusRequest(BaseModel):
    """Request body validator for status check endpoint"""

    vendor: str
    model: str
    architecture: str
    board: BoardValidator
    os: OSValidator
    bios: BiosValidator | None = None
    chassis: ChassisValidator | None = None
    processor: ProcessorValidator
    pci_peripherals: list[PCIPeripheralValidator] = []
    usb_peripherals: list[USBPeripheralValidator] = []
