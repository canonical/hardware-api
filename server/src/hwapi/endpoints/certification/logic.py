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

from hwapi.data_models import models, repository
from hwapi.data_models.data_validators import (
    BiosValidator,
    BoardValidator,
    ProcessorValidator,
)


def find_board(db: Session, board_data: BoardValidator) -> models.Device | None:
    """
    Find the board device based on the given board data.

    Returns ``None`` if the board is not found. Board part number
    variants of the same platform can still resolve via the platform fallback.
    """
    return repository.get_board(db, board_data.manufacturer, board_data.product_name)


def find_bioses(db: Session, bios_data: BiosValidator) -> list[models.Bios]:
    """
    Find the BIOS list based on the given BIOS data.

    Returns an empty list if no matching BIOS is found. No exact matches is not
    a failure, as BIOS updates should still match.
    """
    bios_list = repository.get_bios_list(db, bios_data.vendor, bios_data.version)
    return list(bios_list)


def find_certified_machine(
    db: Session,
    arch: str,
    board: models.Device | None,
    bios_list: list[models.Bios],
    vendor: str,
    model: str,
) -> models.Machine:
    """Find a certified machine matching the request.

    First attempts a match on architecture plus the (optional) board and BIOS.
    A machine can still be found when the board part number or BIOS version has
    drifted from the certified report.

    If the relaxed hardware match fails (e.g. the board could not be
    identified at all), falls back to matching by vendor and model (platform).

    :raises ValueError: If no certified machine matches.
    """
    bios_ids = [bios.id for bios in bios_list] if bios_list else []
    machine = repository.get_machine_with_same_hardware_params(
        db, arch, board, bios_ids
    )
    if machine is None:
        machine = repository.get_machine_by_vendor_and_model(db, arch, vendor, model)
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
