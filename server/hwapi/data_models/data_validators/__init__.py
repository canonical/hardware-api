# Copyright 2024 Canonical Ltd.
# All rights reserved.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
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
"""Data validator models for machine specific components"""


from hwapi.data_models.data_validators.devices import (
    AudioValidator,
    BiosValidator,
    BoardValidator,
    ChassisValidator,
    GPUValidator,
    NetworkAdapterValidator,
    PCIPeripheralValidator,
    ProcessorValidator,
    VideoCaptureValidator,
    WirelessAdapterValidator,
    USBPeripheralValidator,
)

from hwapi.data_models.data_validators.software import (
    KernelPackageValidator,
    OSValidator,
)


__all__ = [
    "AudioValidator",
    "BiosValidator",
    "BoardValidator",
    "ChassisValidator",
    "GPUValidator",
    "KernelPackageValidator",
    "NetworkAdapterValidator",
    "OSValidator",
    "PCIPeripheralValidator",
    "ProcessorValidator",
    "VideoCaptureValidator",
    "WirelessAdapterValidator",
    "USBPeripheralValidator",
]
