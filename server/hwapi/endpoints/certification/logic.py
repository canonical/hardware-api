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
"""The algorithms for determining certification status"""

from sqlalchemy.orm import Session

from hwapi.data_models import repository, models
from hwapi.data_models.data_validators import (
    BoardValidator,
    BiosValidator,
    ProcessorValidator,
)


def find_board(db: Session, board_data: BoardValidator) -> models.Device:
    """
    Find the board device based on the given board data.
    Raises ValueError if the board is not found.
    """
    board = repository.get_board(db, board_data.manufacturer, board_data.product_name)
    if not board:
        raise ValueError("Hardware not certified: Board not found")
    return board


def find_bioses(db: Session, bios_data: BiosValidator) -> list[models.Bios]:
    """
    Find the BIOS list based on the given BIOS data.
    Raises ValueError if no matching BIOS is found.
    """
    bios_list = repository.get_bios_list(db, bios_data.vendor, bios_data.version)
    if not bios_list:
        raise ValueError("Hardware not certified: BIOS not found")
    return list(bios_list)


def find_certified_machine(
    db: Session, arch: str, board: models.Device, bios_list: list[models.Bios]
) -> models.Machine:
    bios_ids = [bios.id for bios in bios_list] if bios_list else []
    machine = repository.get_machine_with_same_hardware_params(
        db, arch, board, bios_ids
    )
    if not machine:
        raise ValueError("No certified machine matches the hardware specifications")
    return machine


def check_cpu_compatibility(
    db: Session, machine: models.Machine, cpu_from_request: ProcessorValidator
) -> bool:
    """
    Check whether the machine has a CPU with the same codename as the cpu_id matching
    codename. If codename is not found, check if the CPU model (version) matches.
    """
    cpu = repository.get_cpu_for_machine(db, machine.id)
    if cpu is None:
        return False

    if cpu_from_request.identifier is None:
        return cpu.version == cpu_from_request.version

    # CPU ID must be complete to check the compatibility
    if len(cpu_from_request.identifier) < 3:
        return False
    cpu_id_hex = (
        f"0x{cpu_from_request.identifier[2]:x}"
        f"{cpu_from_request.identifier[1]:02x}"
        f"{cpu_from_request.identifier[0]:02x}"
    )
    cpu_id_object = repository.get_cpu_id_object(db, cpu_id_hex)
    target_codename = cpu_id_object.codename if cpu_id_object is not None else "Unknown"
    return cpu.codename == target_codename
