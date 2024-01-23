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


class SingleDeviceType(str, Enum):
    """
    Device types that appear as one unit on a machine
    """

    CHASSIS = "CHASSIS"
    BIOS = "BIOS"
    BOARD = "BOARD"
    SYSTEM = "SYSTEM"


class DeviceType(str, Enum):
    AUDIO = "AUDIO"
    BLUETOOTH = "BLUETOOTH"
    CAPTURE = "CAPTURE"
    CARDREADER = "CARDREADER"
    CDROM = "CDROM"
    DISK = "DISK"
    EFI = "EFI"
    HIDRAW = "HIDRAW"
    KEYBOARD = "KEYBOARD"
    MOUSE = "MOUSE"
    NETWORK = "NETWORK"
    OTHER = "OTHER"
    PRINTER = "PRINTER"
    CPU = "CPU"
    SOCKET = "SOCKET"
    TOUCHPAD = "TOUCHPAD"
    TOUCHSCREEN = "TOUCHSCREEN"
    USB = "USB"
    GPU = "GPU"
    WIFI = "WIFI"
    WWAN = "WWAN"


class CertificationStatus(str, Enum):
    CERTIFIED = "Certified"
    PARTIALLY_CERTIFIED = "Partially Certified"
    NOT_SEEN = "Not Seen"
