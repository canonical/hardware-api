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
    db: Session, board_data: BoardValidator, bios_data: BiosValidator | None
) -> tuple[models.Device, models.Bios | None]:
    """
    A function to get "main hardware components" like board and bios. Can be extended
    in future
    """
    board = repository.get_board(
        db, board_data.manufacturer, board_data.product_name, board_data.version
    )
    if not board:
        raise ValueError("Hardware not certified")
    if bios_data:
        bios = repository.get_bios(
            db, bios_data.vendor, bios_data.version, bios_data.firmware_revision
        )
        if not bios:
            raise ValueError("Hardware not certified")
        return board, bios
    return board, None


def find_certified_machine(
    db: Session, arch: str, board: models.Device, bios: models.Bios | None
) -> models.Machine:
    machine = repository.get_machine_with_same_hardware_params(db, arch, board, bios)
    if not machine:
        raise ValueError("No certified machine matches the hardware specifications")
    return machine


def check_cpu_compatibility(
    db: Session, machine: models.Machine, cpu_id: list[int]
) -> bool:
    """
    Check whether the machine has a CPU with the same codename as the cpu_id matching
    codename
    """
    cpu = repository.get_cpu_for_machine(db, machine.id)
    if cpu is None:
        return False
    # CPU ID must be complete to check the compatibility
    if len(cpu_id) < 3:
        return False
    cpu_id_hex = f"0x{cpu_id[2]:x}{cpu_id[1]:02x}{cpu_id[0]:02x}"
    cpu_id_object = repository.get_cpu_id_object(db, cpu_id_hex)
    target_codename = cpu_id_object.codename if cpu_id_object is not None else "Unknown"
    return cpu.codename == target_codename
