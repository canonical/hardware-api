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


class CertificationStatus(str, Enum):
    CERTIFIED = "Certified"
    NOT_SEEN = "Not Seen"
    RELATED_CERTIFIED_SYSTEM_EXISTS = "Related Certified System Exists"
    CERTIFIED_IMAGE_EXISTS = "Certified Image Exists"


class DeviceStatus(str, Enum):
    KNOWN_WORKING = "known-working"
    KNOWN_BREAKING = "known-breaking"
    UNKNOWN = "unknown"


class BusType(str, Enum):
    apex = "apex"
    ata_device = "ata_device"
    backlight = "backlight"
    block = "block"
    bluetooth = "bluetooth"
    cciss = "cciss"
    ccw = "ccw"
    dmi = "dmi"
    drm = "drm"
    firewire = "firewire"
    gameport = "gameport"
    hid = "hid"
    hidraw = "hidraw"
    i2c = "i2c"
    ide = "ide"
    ieee80211 = "ieee80211"
    infiniband = "infiniband"
    input = "input"
    memstick_host = "memstick_host"
    misc = "misc"
    mmc = "mmc"
    mmc_host = "mmc_host"
    mmc_rpmb = "mmc_rpmb"
    mtd = "mtd"
    nd = "nd"
    net = "net"
    nvme = "nvme"
    pci = "pci"
    platform = "platform"
    pnp = "pnp"
    power_supply = "power_supply"
    ppdev = "ppdev"
    rc = "rc"
    rfkill = "rfkill"
    scsi = "scsi"
    scsi_host = "scsi_host"
    sdio = "sdio"
    serial = "serial"
    serio = "serio"
    sound = "sound"
    tty = "tty"
    usb = "usb"
    usb_device = "usb_device"
    vchiq = "vchiq"
    video4linux = "video4linux"
    virtio = "virtio"
    virtual = "virtual"


class DeviceCategory(str, Enum):
    ACCELEROMETER = "ACCELEROMETER"
    AUDIO = "AUDIO"
    BIOS = "BIOS"
    BLUETOOTH = "BLUETOOTH"
    BMC_NETWORK = "BMC_NETWORK"
    BOARD = "BOARD"
    CANBUS = "CANBUS"
    CAPTURE = "CAPTURE"
    CARDREADER = "CARDREADER"
    CDROM = "CDROM"
    CHASSIS = "CHASSIS"
    DISK = "DISK"
    EFI = "EFI"
    FIREWIRE = "FIREWIRE"
    FLOPPY = "FLOPPY"
    HIDRAW = "HIDRAW"
    IDE = "IDE"
    INFINIBAND = "INFINIBAND"
    KEYBOARD = "KEYBOARD"
    MMAL = "MMAL"
    MODEM = "MODEM"
    MOUSE = "MOUSE"
    NETWORK = "NETWORK"
    OTHER = "OTHER"
    PRINTER = "PRINTER"
    PROCESSOR = "PROCESSOR"
    RAID = "RAID"
    SCSI = "SCSI"
    SOCKET = "SOCKET"
    SOCKETCAN = "SOCKETCAN"
    SYSTEM = "SYSTEM"
    TOUCH = "TOUCH"
    TOUCHPAD = "TOUCHPAD"
    TOUCHSCREEN = "TOUCHSCREEN"
    TPU = "TPU"
    USB = "USB"
    VIDEO = "VIDEO"
    WATCHDOG = "WATCHDOG"
    WIRELESS = "WIRELESS"
    WWAN = "WWAN"
