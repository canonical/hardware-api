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

from sqlalchemy import Session

from hwapi.data_models.models import Bios, Device
from hwapi.data_models.repository import get_vendor_by_name, get_board, get_bios
from hwapi.data_models.data_validators import BoardValidator, BiosValidator


def check_machine_vendor(db: Session, vendor_name: str) -> bool:
    """Is the machine vendor one with any certified systems?"""
    vendor = get_vendor_by_name(db, vendor_name)
    return vendor is not None


def match_against_main_componenets(
    db: Session, board: BoardValidator, bios: BiosValidator
) -> tuple[Device | None, Bios | None]:
    """Find the motherboard and BIOS that match a certified system"""
    return (
        get_board(db, board.manufacturer, board.product_name, board.version),
        get_bios(db, bios.vendor, bios.version),
    )
