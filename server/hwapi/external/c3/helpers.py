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


def progress_bar(it: int, total: int):
    """
    Print progress BAR as percentage of it out of total

    :it: the current number of items
    :total: the total number of items
    """
    fillwith = "#"
    dec = 1
    leng = 50
    percent = f"{100 * (it / total):.{dec}f}"
    fill_length = int(leng * it // total)
    prog_bar = fillwith * fill_length + "-" * (leng - fill_length)
    print(f"\rProgress |{prog_bar}| {percent}% Complete", end="")
