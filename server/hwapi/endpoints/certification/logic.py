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
"""The algorithms for determining certification status"""


from sqlalchemy.orm import Session

from hwapi.data_models import repository, models
from hwapi.data_models.data_validators import BoardValidator, BiosValidator


def find_main_hardware_components(
    db: Session, board_data: BoardValidator, bios_data: BiosValidator
) -> tuple[models.Device, models.Bios]:
    """
    A function to get "main hardware components" like board and bios. Can be extended
    in future
    """
    board = repository.get_board(
        db, board_data.manufacturer, board_data.product_name, board_data.version
    )
    bios = repository.get_bios(
        db, bios_data.vendor, bios_data.version, bios_data.firmware_revision
    )
    if not board or not bios:
        raise ValueError("Hardware not certified")
    return board, bios


def find_certified_machine(
    db: Session, arch: str, board: models.Device, bios: models.Bios
) -> models.Machine:
    machine = repository.get_machine_with_same_hardware_params(db, arch, board, bios)
    if not machine:
        raise ValueError("No certified machine matches the hardware specifications")
    return machine


def check_cpu_compatibility(
    db: Session, machine: models.Machine, target_codename: str
) -> bool:
    cpu = repository.get_cpu_for_machine(db, machine.id)
    return bool(cpu and cpu.codename.lower() == target_codename.lower())
