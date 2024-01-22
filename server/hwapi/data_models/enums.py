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


from enum import Enum


class DeviceType(str, Enum):
    AUDIO = "AUDIO"
    BIOS = "BIOS"
    BLUETOOTH = "BLUETOOTH"
    BOARD = "BOARD"
    CHASSIS = "CHASSIS"
    CPU = "PROCESSOR"
    DISK = "DISK"
    GPU = "GPU"
    NETWORK_ADAPTER = "NETWORK"
    VIDEO = "VIDEO"
    WIFI_ADAPTER = "WIRELESS"


class CertificationStatus(str, Enum):
    CERTIFIED = "Certified"
    PARTIALLY_CERTIFIED = "Partially Certified"
    NOT_SEEN = "Not Seen"