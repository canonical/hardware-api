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

from datetime import datetime, date
from pydantic import BaseModel

from hwapi.data_models.enums import BusType, DeviceCategory


class Release(BaseModel):
    codename: str
    release: str
    release_date: date
    supported_until: date
    i_version: int


class Bios(BaseModel):
    name: str
    vendor: str
    version: str
    firmware_type: str


class Device(BaseModel):
    name: str | None = None
    subproduct_name: str | None = None
    vendor: str
    device_type: str | None
    bus: BusType
    identifier: str
    subsystem: str | None
    version: str | None
    category: DeviceCategory | None
    codename: str | None = None


class PublicCertificate(BaseModel):
    canonical_id: str
    vendor: str
    platform: str
    configuration: str
    created_at: datetime
    completed: datetime
    name: str
    release: Release
    architecture: str | None
    kernel_version: str | None
    bios: Bios | None
    firmware_revision: str | None


class PublicDeviceInstance(BaseModel):
    machine_canonical_id: str
    certificate_name: str
    device: Device
    driver_name: str
    cpu_codename: str
