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


from fastapi import APIRouter

from hwapi.data_models.models_dto import SystemInfoDTO, DeviceDTO
from hwapi.data_models.enums import CertificationStatus, DeviceType
from hwapi.endpoints.response_dto import (
    CertificationResponse,
    PartialCertificationResponse,
)


router = APIRouter()


def check_certification_status(
    system_info: SystemInfoDTO,
) -> tuple[CertificationStatus, list[DeviceDTO]]:
    # Example logic
    certified_vendors = ["Vendor1", "Vendor2"]
    certified_models = {"Vendor1": ["Model1", "Model2"], "Vendor2": ["Model3"]}

    if system_info.vendor not in certified_vendors:
        return CertificationStatus.NOT_SEEN, []

    if system_info.model in certified_models.get(system_info.vendor, []):
        return CertificationStatus.CERTIFIED, []

    certified_components = []
    for device in system_info.devices:
        if device.device_type in [DeviceType.BOARD, DeviceType.CPU, DeviceType.GPU]:
            certified_components.append(device)

    if not certified_components:
        return CertificationStatus.NOT_SEEN, []

    return CertificationStatus.PARTIALLY_CERTIFIED, certified_components


@router.post(
    "/status", response_model=CertificationResponse | PartialCertificationResponse
)
def check_certification(system_info: SystemInfoDTO):
    status, components = check_certification_status(system_info)

    if status != CertificationStatus.PARTIALLY_CERTIFIED:
        return {"detail": status}
    return {"detail": status, "certified_components": components}
